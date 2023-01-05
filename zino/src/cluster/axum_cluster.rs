use axum::{
    body::{Bytes, Full},
    error_handling::HandleErrorLayer,
    extract::{rejection::LengthLimitError, DefaultBodyLimit},
    http::{self, StatusCode},
    middleware, routing, BoxError, Router, Server,
};
use futures::future;
use std::{
    collections::HashMap,
    convert::Infallible,
    env, io,
    net::SocketAddr,
    path::Path,
    sync::{Arc, LazyLock},
    time::{Duration, Instant},
};
use tokio::runtime::Builder;
use tower::{
    timeout::{error::Elapsed, TimeoutLayer},
    ServiceBuilder,
};
use tower_http::{
    add_extension::AddExtensionLayer,
    compression::CompressionLayer,
    services::{ServeDir, ServeFile},
};
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
                let shared_state = State::shared();
                let app_env = shared_state.env();
                tracing::info!("load config.{app_env}.toml");

                let listeners = shared_state.listeners();
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

                    let state = Arc::new(State::default());
                    app = app
                        .fallback_service(tower::service_fn(|_| async {
                            let res = Response::new(StatusCode::NOT_FOUND);
                            Ok::<http::Response<Full<Bytes>>, Infallible>(res.into())
                        }))
                        .layer(
                            ServiceBuilder::new()
                                .layer(LazyLock::force(
                                    &crate::middleware::tower_tracing::TRACING_MIDDLEWARE,
                                ))
                                .layer(LazyLock::force(
                                    &crate::middleware::tower_cors::CORS_MIDDLEWARE,
                                ))
                                .layer(middleware::from_fn(
                                    crate::middleware::axum_context::request_context,
                                ))
                                .layer(DefaultBodyLimit::disable())
                                .layer(AddExtensionLayer::new(state))
                                .layer(CompressionLayer::new())
                                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                                    let status_code = if err.is::<Elapsed>() {
                                        StatusCode::REQUEST_TIMEOUT
                                    } else if err.is::<LengthLimitError>() {
                                        StatusCode::PAYLOAD_TOO_LARGE
                                    } else {
                                        StatusCode::INTERNAL_SERVER_ERROR
                                    };
                                    let res = Response::new(status_code);
                                    Ok::<http::Response<Full<Bytes>>, Infallible>(res.into())
                                }))
                                .layer(TimeoutLayer::new(Duration::from_secs(10))),
                        );

                    let addr = listener
                        .parse()
                        .inspect(|addr| tracing::info!(env = app_env, "listen on {addr}"))
                        .unwrap_or_else(|_| panic!("invalid socket address: {listener}"));
                    Server::bind(&addr)
                        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
                });
                for result in future::join_all(servers).await {
                    if let Err(err) = result {
                        tracing::error!("server error: {err}");
                    }
                }
            });
        Ok(())
    }
}
