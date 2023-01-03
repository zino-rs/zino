use axum::{
    body::{Bytes, Full},
    http::{self, StatusCode},
    routing, Router, Server,
};
use futures::future;
use std::{
    collections::HashMap, convert::Infallible, env, io, net::SocketAddr, path::Path, time::Instant,
};
use tokio::runtime::Builder;
use tower::layer;
use tower_http::services::{ServeDir, ServeFile};
use zino_core::{Application, Response, State};

/// An HTTP server cluster for `axum`.
pub struct AxumCluster {
    /// Start time.
    start_time: Instant,
    /// Routes.
    routes: HashMap<&'static str, Router>,
}

impl Application for AxumCluster {
    /// Router.
    type Router = HashMap<&'static str, Router>;

    /// Creates a new application.
    fn new() -> Self {
        Self {
            start_time: Instant::now(),
            routes: HashMap::new(),
        }
    }

    /// Registers the router.
    fn register(mut self, routes: Self::Router) -> Self {
        self.routes = routes;
        self
    }

    /// Returns the start time.
    #[inline]
    fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Runs the application.
    fn run(self) -> io::Result<()> {
        let current_dir = env::current_dir().unwrap();
        let project_dir = Path::new(&current_dir);
        let public_dir = project_dir.join("./public");
        let static_site_dir = if public_dir.exists() {
            public_dir
        } else {
            project_dir.join("../public")
        };
        let index_file = static_site_dir.join("./index.html");
        let serve_file_service = routing::get_service(ServeFile::new(index_file)).handle_error(
            |err: io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {err}"),
                )
            },
        );
        let serve_dir_service = routing::get_service(ServeDir::new(static_site_dir)).handle_error(
            |err: io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {err}"),
                )
            },
        );
        Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(async {
                let routes = self.routes;
                let listeners = State::shared().listeners();
                let servers = listeners.iter().map(|listener| {
                    let mut app = Router::new()
                        .route_service("/", serve_file_service.clone())
                        .nest_service("/public", serve_dir_service.clone())
                        .route("/sse", routing::get(crate::endpoint::axum_sse::sse_handler))
                        .route(
                            "/websocket",
                            routing::get(crate::endpoint::axum_websocket::websocket_handler),
                        );
                    for (path, route) in &routes {
                        app = app.nest(path, route.clone());
                    }
                    app = app
                        .fallback_service(tower::service_fn(|_| async {
                            let res = Response::new(StatusCode::NOT_FOUND);
                            Ok::<http::Response<Full<Bytes>>, Infallible>(res.into())
                        }))
                        .layer(layer::layer_fn(
                            crate::middleware::axum_context::ContextMiddleware::new,
                        ));

                    let addr = listener
                        .parse()
                        .inspect(|addr| println!("listen on {addr}"))
                        .unwrap_or_else(|_| panic!("invalid socket address: {listener}"));
                    Server::bind(&addr)
                        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                });
                for result in future::join_all(servers).await {
                    if let Err(err) = result {
                        eprintln!("server error: {err}");
                    }
                }
            });
        Ok(())
    }
}
