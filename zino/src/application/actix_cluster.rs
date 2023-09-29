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
use std::{fs, path::PathBuf};
use utoipa_rapidoc::RapiDoc;
use zino_core::{
    application::Application,
    extension::TomlTableExt,
    response::Response,
    schedule::{AsyncCronJob, Job, JobScheduler},
};

/// An HTTP server cluster for `actix-web`.
#[derive(Default)]
pub struct ActixCluster {
    /// Default routes.
    default_routes: Vec<RouterConfigure>,
    /// Named routes.
    named_routes: Vec<(&'static str, Vec<RouterConfigure>)>,
}

impl Application for ActixCluster {
    type Routes = Vec<RouterConfigure>;

    fn register(mut self, routes: Self::Routes) -> Self {
        self.default_routes = routes;
        self
    }

    fn register_with(mut self, server_name: &'static str, routes: Self::Routes) -> Self {
        self.named_routes.push((server_name, routes));
        self
    }

    fn run(self, async_jobs: Vec<(&'static str, AsyncCronJob)>) {
        let runtime = Runtime::new().expect("fail to build Tokio runtime for `ActixCluster`");
        let mut scheduler = JobScheduler::new();
        for (cron_expr, exec) in async_jobs {
            scheduler.add(Job::new_async(cron_expr, exec));
        }
        runtime.spawn(async move {
            loop {
                scheduler.tick_async().await;

                // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                rt::time::sleep(scheduler.time_till_next_job()).await;
            }
        });

        runtime.block_on(async {
            let default_routes = self.default_routes.leak() as &'static [_];
            let named_routes = self.named_routes.leak() as &'static [_];
            let app_state = Self::shared_state();
            let app_name = Self::name();
            let app_version = Self::version();
            let app_env = app_state.env();
            let listeners = app_state.listeners();
            let has_debug_server = listeners.iter().any(|listener| listener.0 == "debug");
            let servers = listeners.into_iter().map(|listener| {
                let server_name = listener.0;
                let addr = listener.1;
                tracing::warn!(
                    server_name = server_name.as_ref(),
                    app_env,
                    app_name,
                    app_version,
                    "listen on {addr}",
                );

                // Server config
                let project_dir = Self::project_dir();
                let default_public_dir = project_dir.join("public");
                let mut public_route_prefix = "/public";
                let mut public_dir = PathBuf::new();
                let mut body_limit = 100 * 1024 * 1024; // 100MB
                if let Some(config) = app_state.get_config("server") {
                    if let Some(limit) = config.get_usize("body-limit") {
                        body_limit = limit;
                    }
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
                } else {
                    public_dir = default_public_dir;
                }

                HttpServer::new(move || {
                    let index_file_handler = web::get()
                        .to(|| async { NamedFile::open_async("./public/index.html").await });
                    let static_files = Files::new(public_route_prefix, public_dir.clone())
                        .show_files_listing()
                        .index_file("index.html")
                        .prefer_utf8(true)
                        .default_handler(fn_service(|req: ServiceRequest| async {
                            let (req, _) = req.into_parts();
                            let file = NamedFile::open_async("./public/404.html").await?;
                            let res = file.into_response(&req);
                            Ok(ServiceResponse::new(req, res))
                        }));
                    let mut app = App::new()
                        .route("/", index_file_handler)
                        .service(static_files)
                        .default_service(web::to(|req: Request| async {
                            let res = Response::new(StatusCode::NOT_FOUND);
                            ActixResponse::from(res).respond_to(&req.into())
                        }));
                    for route in default_routes {
                        app = app.configure(route);
                    }
                    for (name, routes) in named_routes {
                        if name == &server_name || server_name == "debug" {
                            for route in routes {
                                app = app.configure(route);
                            }
                        }
                    }

                    // Render OpenAPI docs.
                    let docs_server_name = if has_debug_server { "debug" } else { "main" };
                    if docs_server_name == server_name {
                        if let Some(config) = app_state.get_config("openapi") {
                            if config.get_bool("show-docs") != Some(false) {
                                // If the `spec-url` has been configured, the user should
                                // provide the generated OpenAPI object with a derivation.
                                let mut rapidoc = if let Some(url) = config.get_str("spec-url") {
                                    RapiDoc::new(url)
                                } else {
                                    RapiDoc::with_openapi("/api-docs/openapi.json", Self::openapi())
                                };
                                if let Some(route) = config.get_str("rapidoc-route") {
                                    rapidoc = rapidoc.path(route);
                                } else {
                                    rapidoc = rapidoc.path("/rapidoc");
                                }
                                if let Some(custom_html) = config.get_str("custom-html") &&
                                    let Ok(html) = fs::read_to_string(project_dir.join(custom_html))
                                {
                                    rapidoc = rapidoc.custom_html(html.leak());
                                }
                                app = app.service(rapidoc);
                            }
                        } else {
                            let rapidoc =
                                RapiDoc::with_openapi("/api-docs/openapi.json", Self::openapi())
                                    .path("/rapidoc");
                            app = app.service(rapidoc);
                        }
                    }

                    app.app_data(FormConfig::default().limit(body_limit))
                        .app_data(JsonConfig::default().limit(body_limit))
                        .app_data(PayloadConfig::default().limit(body_limit))
                        .wrap(Compress::default())
                        .wrap(middleware::RequestContextInitializer::default())
                        .wrap(middleware::tracing_middleware())
                        .wrap(middleware::cors_middleware())
                        .wrap(middleware::ETagFinalizer::default())
                })
                .bind(addr)
                .unwrap_or_else(|err| panic!("fail to create an HTTP server: {err}"))
                .run()
            });
            for result in futures::future::join_all(servers).await {
                if let Err(err) = result {
                    tracing::error!("server error: {err}");
                }
            }
        });
    }
}
