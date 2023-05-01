use crate::RouterConfigure;
use actix_web::{rt::System, App, HttpServer};
use zino_core::{application::Application, schedule::AsyncCronJob, state::State};

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

    fn run(self, _async_jobs: Vec<(&'static str, AsyncCronJob)>) {
        let routes = self.routes;
        let app_state = State::default();
        let app_env = app_state.env();
        let listeners = app_state.listeners();
        listeners.iter().for_each(|listener| {
            tracing::warn!(env = app_env, "listen on {listener}");
        });
        System::new()
            .block_on(
                HttpServer::new(move || {
                    let mut app = App::new();
                    for route in &routes {
                        app = app.configure(route);
                    }
                    app
                })
                .bind(listeners.as_slice())
                .unwrap_or_else(|err| panic!("fail to create an HTTP server: {err}"))
                .run(),
            )
            .unwrap_or_else(|err| panic!("fail to build Actix runtime: {err}"))
    }
}
