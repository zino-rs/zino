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
    sync::LazyLock,
    time::Duration,
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
use tracing::Level;
use tracing_subscriber::fmt::{time, writer::MakeWriterExt};
use zino_core::{Application, AsyncCronJob, DateTime, Job, JobScheduler, Map, Response, State};

/// An HTTP server cluster for `axum`.
pub struct AxumCluster {
    /// Routes.
    routes: HashMap<&'static str, Router>,
}

impl Application for AxumCluster {
    /// Router.
    type Router = Router;

    /// Creates a new application.
    fn new() -> Self {
        let app_env = Self::env();
        let is_dev = app_env == "dev";
        let mut env_filter = if is_dev {
            "sqlx=trace,tower_http=trace,zino=trace,zino_core=trace"
        } else {
            "sqlx=warn,tower_http=info,zino=info,zino_core=info"
        };
        let mut display_target = true;
        let mut display_filename = false;
        let mut display_line_number = false;
        let mut display_thread_names = false;
        let mut display_span_list = false;
        let display_current_span = true;

        if let Some(tracing) = Self::config().get("tracing").and_then(|t| t.as_table()) {
            if let Some(filter) = tracing.get("filter").and_then(|t| t.as_str()) {
                env_filter = filter;
            }
            display_target = tracing
                .get("display-target")
                .and_then(|t| t.as_bool())
                .unwrap_or(true);
            display_filename = tracing
                .get("display-filename")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            display_line_number = tracing
                .get("display-line-number")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            display_thread_names = tracing
                .get("display-thread-names")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
            display_span_list = tracing
                .get("display-span-list")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);
        }

        let stderr = io::stderr.with_max_level(Level::WARN);
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(display_target)
            .with_file(display_filename)
            .with_line_number(display_line_number)
            .with_thread_names(display_thread_names)
            .with_span_list(display_span_list)
            .with_current_span(display_current_span)
            .with_timer(time::LocalTime::rfc_3339())
            .map_writer(move |w| stderr.or_else(w))
            .init();

        Self {
            routes: HashMap::new(),
        }
    }

    /// Returns a reference to the shared application state.
    #[inline]
    fn shared() -> &'static State {
        LazyLock::force(&SHARED_CLUSTER_STATE)
    }

    /// Registers routes.
    fn register(mut self, routes: HashMap<&'static str, Self::Router>) -> Self {
        self.routes = routes;
        self
    }

    /// Runs the application.
    fn run(self, async_jobs: HashMap<&'static str, AsyncCronJob>) {
        let app_state = State::default();
        let app_env = app_state.env();
        tracing::info!("load config.{app_env}.toml");

        let runtime = Builder::new_multi_thread().enable_all().build().unwrap();
        let mut scheduler = JobScheduler::new();
        for (cron_expr, exec) in async_jobs {
            scheduler.add(Job::new_async(cron_expr, exec));
        }
        runtime.spawn(async move {
            loop {
                scheduler.tick_async().await;

                // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                tokio::time::sleep(scheduler.time_till_next_job()).await;
            }
        });

        let current_dir = env::current_dir().unwrap();
        let project_dir = Path::new(&current_dir);
        let public_dir = project_dir.join("./public");
        let static_site_dir = if public_dir.exists() {
            public_dir
        } else {
            project_dir.join("../public")
        };
        let index_file = static_site_dir.join("./index.html");
        let internal_server_error_handler = |err: io::Error| async move {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {err}"),
            )
        };
        let serve_file_service = routing::get_service(ServeFile::new(index_file))
            .handle_error(internal_server_error_handler);
        let serve_dir_service = routing::get_service(ServeDir::new(static_site_dir))
            .handle_error(internal_server_error_handler);

        runtime.block_on(async {
            let routes = self.routes;
            let listeners = app_state.listeners();
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

                let state = app_state.clone();
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
                            // be consistent with the default acquire timeout of `PoolOptions`.
                            .layer(TimeoutLayer::new(Duration::from_secs(30))),
                    );

                let addr = listener
                    .parse()
                    .inspect(|addr| tracing::info!(env = app_env, "listen on {addr}"))
                    .unwrap_or_else(|_| panic!("invalid socket address: {listener}"));
                Server::bind(&addr).serve(app.into_make_service_with_connect_info::<SocketAddr>())
            });
            for result in future::join_all(servers).await {
                if let Err(err) = result {
                    tracing::error!("server error: {err}");
                }
            }
        });
    }
}

/// Shared cluster state.
static SHARED_CLUSTER_STATE: LazyLock<State> = LazyLock::new(|| {
    let mut state = State::default();
    let config = state.config();
    let app_name = config
        .get("name")
        .and_then(|t| t.as_str())
        .expect("the `name` field should be specified");
    let app_version = config
        .get("version")
        .and_then(|t| t.as_str())
        .expect("the `version` field should be specified");

    let mut data = Map::new();
    data.insert("app_name".to_string(), app_name.into());
    data.insert("app_version".to_string(), app_version.into());
    data.insert("cluster_start_at".to_string(), DateTime::now().into());
    state.set_data(data);
    state
});
