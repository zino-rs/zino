use crate::{middleware, ActixResponse, Request, RouterConfigure};
use actix_files::{Files, NamedFile};
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    http::StatusCode,
    middleware::Compress,
    rt::{self, Runtime},
    web::{self, FormConfig, JsonConfig, PayloadConfig},
    App, HttpServer, Responder,
};
use std::{fs, path::PathBuf, time::Duration};
use utoipa_rapidoc::RapiDoc;
use zino_core::{
    application::{Application, Plugin, ServerTag},
    extension::TomlTableExt,
    response::Response,
    schedule::AsyncScheduler,
};

/// An HTTP server cluster for `actix-web`.
#[derive(Default)]
pub struct ActixCluster {
    /// Custom plugins.
    custom_plugins: Vec<Plugin>,
    /// Default routes.
    default_routes: Vec<RouterConfigure>,
    /// Tagged routes.
    tagged_routes: Vec<(ServerTag, Vec<RouterConfigure>)>,
}

impl Application for ActixCluster {
    type Routes = Vec<RouterConfigure>;

    fn register(mut self, routes: Self::Routes) -> Self {
        self.default_routes = routes;
        self
    }

    fn register_with(mut self, server_tag: ServerTag, routes: Self::Routes) -> Self {
        self.tagged_routes.push((server_tag, routes));
        self
    }

    fn add_plugin(mut self, plugin: Plugin) -> Self {
        self.custom_plugins.push(plugin);
        self
    }

    fn run_with<T: AsyncScheduler + Send + 'static>(self, mut scheduler: T) {
        let runtime = Runtime::new().expect("fail to build Tokio runtime for `ActixCluster`");
        let app_env = Self::env();
        runtime.block_on(async {
            Self::load().await;
            super::load_plugins(self.custom_plugins, app_env).await;
        });
        if scheduler.is_ready() {
            runtime.spawn(async move {
                loop {
                    scheduler.tick().await;

                    // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                    rt::time::sleep(scheduler.time_till_next_job()).await;
                }
            });
        }

        runtime.block_on(async {
            let default_routes = self.default_routes.leak() as &'static [_];
            let tagged_routes = self.tagged_routes.leak() as &'static [_];
            let app_state = Self::shared_state();
            let app_name = Self::name();
            let app_version = Self::version();
            let app_domain = Self::domain();
            let listeners = app_state.listeners();
            let has_debug_server = listeners.iter().any(|listener| listener.0.is_debug());
            let servers = listeners.into_iter().map(|listener| {
                let server_tag = listener.0;
                let addr = listener.1;
                tracing::warn!(
                    server_tag = server_tag.as_str(),
                    app_env = app_env.as_str(),
                    app_name,
                    app_version,
                    zino_version = env!("CARGO_PKG_VERSION"),
                    "listen on `{addr}`",
                );

                // Server config
                let project_dir = Self::project_dir();
                let default_public_dir = project_dir.join("public");
                let mut public_route_prefix = "/public";
                let mut public_dir = PathBuf::new();
                let mut backlog = 2048; // Maximum number of pending connections
                let mut max_connections = 25000; // Maximum number of concurrent connections
                let mut body_limit = 128 * 1024 * 1024; // 128MB
                let mut request_timeout = Duration::from_secs(60); // 60 seconds
                if let Some(config) = app_state.get_config("server") {
                    if let Some(dir) = config.get_str("page-dir") {
                        public_route_prefix = "/page";
                        public_dir.push(dir);
                    } else if let Some(dir) = config.get_str("public-dir") {
                        public_dir.push(dir);
                    } else {
                        public_dir = default_public_dir;
                    }
                    if let Some(route_prefix) = config.get_str("public-route-prefix") {
                        public_route_prefix = route_prefix;
                    }
                    if let Some(value) = config.get_u32("backlog") {
                        backlog = value;
                    }
                    if let Some(value) = config.get_usize("max-connections") {
                        max_connections = value;
                    }
                    if let Some(limit) = config.get_usize("body-limit") {
                        body_limit = limit;
                    }
                    if let Some(timeout) = config.get_duration("request-timeout") {
                        request_timeout = timeout;
                    }
                } else {
                    public_dir = default_public_dir;
                }

                HttpServer::new(move || {
                    let mut app = App::new().default_service(web::to(|req: Request| async {
                        let res = Response::new(StatusCode::NOT_FOUND);
                        ActixResponse::from(res).respond_to(&req.into())
                    }));
                    if public_dir.exists() {
                        let index_file = public_dir.join("index.html");
                        let favicon_file = public_dir.join("favicon.ico");
                        if index_file.exists() {
                            let index_file_handler = web::get().to(move || async {
                                NamedFile::open_async("./public/index.html").await
                            });
                            app = app.route("/", index_file_handler);
                        }
                        if favicon_file.exists() {
                            let favicon_file_handler = web::get().to(|| async {
                                NamedFile::open_async("./public/favicon.ico").await
                            });
                            app = app.route("/favicon.ico", favicon_file_handler);
                        }

                        let mut static_files = Files::new(public_route_prefix, public_dir.clone())
                            .show_files_listing()
                            .index_file("index.html")
                            .prefer_utf8(true);
                        let not_found_file = public_dir.join("404.html");
                        if not_found_file.exists() {
                            static_files = static_files.default_handler(fn_service(
                                |req: ServiceRequest| async {
                                    let (req, _) = req.into_parts();
                                    let file = NamedFile::open_async("./public/404.html").await?;
                                    let res = file.into_response(&req);
                                    Ok(ServiceResponse::new(req, res))
                                },
                            ));
                        }
                        app = app.service(static_files);
                        tracing::info!(
                            "Static pages `{public_route_prefix}/**` are registered for `{addr}`"
                        );
                    }
                    for route in default_routes {
                        app = app.configure(route);
                    }
                    for (tag, routes) in tagged_routes {
                        if tag == &server_tag || server_tag.is_debug() {
                            for route in routes {
                                app = app.configure(route);
                            }
                        }
                    }

                    // OpenAPI docs
                    let is_docs_server = if has_debug_server {
                        server_tag.is_debug()
                    } else {
                        server_tag.is_main()
                    };
                    if is_docs_server {
                        if let Some(config) = app_state.get_config("openapi") {
                            if config.get_bool("show-docs") != Some(false) {
                                // If the `spec-url` has been configured, the user should
                                // provide the generated OpenAPI object with a derivation.
                                let path = config.get_str("rapidoc-route").unwrap_or("/rapidoc");
                                let mut rapidoc = if let Some(url) = config.get_str("spec-url") {
                                    RapiDoc::new(url)
                                } else {
                                    RapiDoc::with_openapi("/api-docs/openapi.json", Self::openapi())
                                };
                                if let Some(custom_html) = config.get_str("custom-html") {
                                    let custom_html_file = project_dir.join(custom_html);
                                    if let Ok(html) = fs::read_to_string(custom_html_file) {
                                        rapidoc = rapidoc.custom_html(html);
                                    }
                                }
                                app = app.service(rapidoc.path(path));
                                tracing::info!(
                                    "RapiDoc router `{path}` is registered for `{addr}`"
                                );
                            }
                        } else {
                            let rapidoc =
                                RapiDoc::with_openapi("/api-docs/openapi.json", Self::openapi())
                                    .path("/rapidoc");
                            app = app.service(rapidoc);
                            tracing::info!("RapiDoc router `/rapidoc` is registered for `{addr}`");
                        }
                    }

                    app.app_data(FormConfig::default().limit(body_limit))
                        .app_data(JsonConfig::default().limit(body_limit))
                        .app_data(PayloadConfig::default().limit(body_limit))
                        .wrap(Compress::default())
                        .wrap(middleware::RequestContextInitializer)
                        .wrap(middleware::tracing_middleware())
                        .wrap(middleware::cors_middleware())
                        .wrap(middleware::ETagFinalizer)
                })
                .server_hostname(app_domain)
                .backlog(backlog)
                .max_connections(max_connections)
                .client_request_timeout(request_timeout)
                .bind(addr)
                .unwrap_or_else(|err| panic!("fail to create an HTTP server: {err}"))
                .run()
            });
            for result in futures::future::join_all(servers).await {
                if let Err(err) = result {
                    tracing::error!("actix server error: {err}");
                }
            }
        });
    }
}
