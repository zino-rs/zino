use crate::{middleware, RouterConfigure};
use actix_files::{Files, NamedFile};
use actix_web::{
    dev::{fn_service, ServiceRequest, ServiceResponse},
    middleware::Compress,
    rt::{self, Runtime},
    web, App, HttpServer,
};
use std::path::PathBuf;
use zino_core::{
    application::Application,
    extension::TomlTableExt,
    schedule::{AsyncCronJob, Job, JobScheduler},
    state::State,
};

/// An HTTP server cluster for `actix-web`.
#[derive(Default)]
pub struct ActixCluster {
    /// Routes.
    routes: Vec<RouterConfigure>,
}

impl Application for ActixCluster {
    type Router = RouterConfigure;

    fn register(mut self, routes: Vec<Self::Router>) -> Self {
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

        // Server config.
        let mut public_dir = PathBuf::new();
        let default_public_dir = Self::project_dir().join("public");
        if let Some(server) = Self::config().get_table("server") {
            if let Some(dir) = server.get_str("public-dir") {
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
                let app_state = State::default();
                let app_env = app_state.env();
                let listeners = app_state.listeners();
                listeners.iter().for_each(|listener| {
                    tracing::warn!(env = app_env, "listen on {listener}");
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
                        .service(static_files);
                    for route in &routes {
                        app = app.configure(route).configure(|cfg| {
                            cfg.app_data(app_state.clone());
                        });
                    }
                    app.wrap(Compress::default())
                        .wrap(middleware::RequestContextInitializer)
                })
                .bind(listeners.as_slice())
                .unwrap_or_else(|err| panic!("fail to create an HTTP server: {err}"))
                .run()
            })
            .unwrap_or_else(|err| panic!("fail to build Actix runtime: {err}"))
    }
}
