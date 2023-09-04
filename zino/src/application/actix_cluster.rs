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
    /// Routes.
    routes: Vec<RouterConfigure>,
}

impl Application for ActixCluster {
    type Routes = Vec<RouterConfigure>;

    fn register(mut self, routes: Self::Routes) -> Self {
        self.routes = routes;
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

        // Server config
        let mut body_limit = 100 * 1024 * 1024; // 100MB
        let mut public_dir = PathBuf::new();
        let default_public_dir = Self::relative_path("public");
        if let Some(server_config) = Self::config().get_table("server") {
            if let Some(limit) = server_config.get_usize("body-limit") {
                body_limit = limit;
            }
            if let Some(dir) = server_config.get_str("public-dir") {
                public_dir.push(dir);
            } else {
                public_dir = default_public_dir;
            }
        } else {
            public_dir = default_public_dir;
        }

        runtime
            .block_on({
                let routes = self.routes;
                let app_state = Self::shared_state();
                let app_name = Self::name();
                let app_version = Self::version();
                let app_env = app_state.env();
                let listeners = app_state.listeners();
                listeners.iter().for_each(|listener| {
                    tracing::warn!(
                        env = app_env,
                        name = app_name,
                        version = app_version,
                        "listen on {listener}",
                    );
                });
                HttpServer::new(move || {
                    let index_file_handler = web::get()
                        .to(move || async { NamedFile::open_async("./public/index.html").await });
                    let static_files = Files::new("/public", public_dir.clone())
                        .show_files_listing()
                        .prefer_utf8(true)
                        .index_file("index.html")
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
                    for route in &routes {
                        app = app.configure(route);
                    }

                    // Render OpenAPI docs.
                    if let Some(openapi_config) = app_state.get_config("openapi") {
                        if openapi_config.get_bool("show-docs") != Some(false) {
                            let mut rapidoc =
                                RapiDoc::with_openapi("/api-docs/openapi.json", Self::openapi())
                                    .path("/rapidoc");
                            if let Some(custom_html) = openapi_config.get_str("custom-html") &&
                                let Ok(html) = fs::read_to_string(Self::relative_path(custom_html))
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

                    app.app_data(FormConfig::default().limit(body_limit))
                        .app_data(JsonConfig::default().limit(body_limit))
                        .app_data(PayloadConfig::default().limit(body_limit))
                        .wrap(Compress::default())
                        .wrap(middleware::RequestContextInitializer::default())
                        .wrap(middleware::tracing_middleware())
                        .wrap(middleware::cors_middleware())
                })
                .bind(listeners.as_slice())
                .unwrap_or_else(|err| panic!("fail to create an HTTP server: {err}"))
                .run()
            })
            .unwrap_or_else(|err| panic!("fail to build Actix runtime: {err}"))
    }
}
