use crate::{AxumResponse, Extractor, middleware};
use axum::{
    BoxError, Router,
    error_handling::HandleErrorLayer,
    extract::{DefaultBodyLimit, rejection::LengthLimitError},
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::from_fn,
};
use std::{any::Any, borrow::Cow, convert::Infallible, fs, net::SocketAddr, time::Duration};
use tokio::{net::TcpListener, runtime::Builder, signal};
use tower::{
    ServiceBuilder,
    timeout::{TimeoutLayer, error::Elapsed},
};
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::{CompressionLayer, predicate::DefaultPredicate},
    decompression::DecompressionLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};
use utoipa_rapidoc::RapiDoc;
use zino_core::{
    LazyLock,
    application::{AppType, Application, Plugin, ServerTag},
    extension::TomlTableExt,
    schedule::AsyncScheduler,
};
use zino_http::response::Response;

/// An HTTP server cluster.
#[derive(Default)]
pub struct Cluster {
    /// Custom plugins.
    custom_plugins: Vec<Plugin>,
    /// Default routes.
    default_routes: Vec<Router>,
    /// Tagged routes.
    tagged_routes: Vec<(ServerTag, Vec<Router>)>,
}

impl Application for Cluster {
    type Routes = Vec<Router>;

    const APP_TYPE: AppType = AppType::Server;

    #[inline]
    fn register(mut self, routes: Self::Routes) -> Self {
        self.default_routes = routes;
        self
    }

    #[inline]
    fn register_with(mut self, server_tag: ServerTag, routes: Self::Routes) -> Self {
        self.tagged_routes.push((server_tag, routes));
        self
    }

    #[inline]
    fn add_plugin(mut self, plugin: Plugin) -> Self {
        self.custom_plugins.push(plugin);
        self
    }

    fn run_with<T: AsyncScheduler + Send + 'static>(self, mut scheduler: T) {
        let runtime = Builder::new_multi_thread()
            .thread_keep_alive(Duration::from_secs(60))
            .thread_stack_size(2 * 1024 * 1024)
            .global_queue_interval(61)
            .enable_all()
            .build()
            .expect("fail to build Tokio runtime for `AxumCluster`");
        let app_env = Self::env();
        runtime.block_on(async {
            #[cfg(feature = "orm")]
            zino_orm::GlobalPool::connect_all().await;
            Self::load().await;
            app_env.load_plugins(self.custom_plugins).await;
        });
        if scheduler.is_ready() {
            if scheduler.is_blocking() {
                runtime.spawn(async move {
                    if let Err(err) = scheduler.run().await {
                        tracing::error!("fail to run the async scheduler: {err}");
                    }
                });
            } else {
                runtime.spawn(async move {
                    loop {
                        scheduler.tick().await;

                        // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                        if let Some(duration) = scheduler.time_till_next_job() {
                            tokio::time::sleep(duration).await;
                        }
                    }
                });
            }
        }

        runtime.block_on(async {
            let default_routes = self.default_routes;
            let tagged_routes = self.tagged_routes;
            let app_state = Self::shared_state();
            let app_name = Self::name();
            let app_version = Self::version();
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
                let mut public_dir = "public";
                let mut public_route_prefix = "/public";
                let mut body_limit = 128 * 1024 * 1024; // 128MB
                let mut request_timeout = Duration::from_secs(60); // 60 seconds
                let mut keep_alive_timeout = 75; // 75 seconds
                if let Some(config) = app_state.get_config("server") {
                    if let Some(dir) = config.get_str("page-dir") {
                        public_dir = dir;
                        public_route_prefix = "/page";
                    } else if let Some(dir) = config.get_str("public-dir") {
                        public_dir = dir;
                    }
                    if let Some(route_prefix) = config.get_str("public-route-prefix") {
                        public_route_prefix = route_prefix;
                    }
                    if let Some(limit) = config.get_usize("body-limit") {
                        body_limit = limit;
                    }
                    if let Some(timeout) = config.get_duration("request-timeout") {
                        request_timeout = timeout;
                    }
                    if let Some(timeout) = config.get_duration("keep-alive-timeout") {
                        keep_alive_timeout = timeout.as_secs();
                    }
                }

                let mut app = Router::new();
                let public_dir = Self::parse_path(public_dir);
                let not_found_file = public_dir.join("404.html");
                if public_dir.exists() {
                    let index_file = public_dir.join("index.html");
                    let favicon_file = public_dir.join("favicon.ico");
                    if index_file.exists() {
                        app = app.route_service("/", ServeFile::new(index_file));
                    }
                    if favicon_file.exists() {
                        app = app.route_service("/favicon.ico", ServeFile::new(favicon_file));
                    }

                    let not_found_file = public_dir.join("404.html");
                    let serve_dir = ServeDir::new(public_dir)
                        .precompressed_gzip()
                        .precompressed_br()
                        .append_index_html_on_directories(true)
                        .not_found_service(ServeFile::new(&not_found_file));
                    let mut serve_dir_route =
                        Router::new().nest_service(public_route_prefix, serve_dir);
                    if public_route_prefix.ends_with("/page") {
                        serve_dir_route =
                            serve_dir_route.layer(from_fn(middleware::serve_static_pages));
                    }
                    app = app.merge(serve_dir_route);
                    tracing::info!(
                        "Static pages `{public_route_prefix}/**` are registered for `{addr}`"
                    );
                }
                if not_found_file.exists() {
                    app = app.fallback_service(ServeFile::new(not_found_file));
                } else {
                    app = app.fallback_service(tower::service_fn(|req| async {
                        let req = Extractor::from(req);
                        let res = Response::new(StatusCode::NOT_FOUND).context(&req);
                        Ok::<AxumResponse, Infallible>(res.into())
                    }));
                }
                for route in &default_routes {
                    app = app.merge(route.clone());
                }
                for (tag, routes) in &tagged_routes {
                    if tag == &server_tag || server_tag.is_debug() {
                        for route in routes {
                            app = app.merge(route.clone());
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
                    let openapi = zino_openapi::openapi();
                    if let Some(config) = app_state.get_config("openapi") {
                        if config.get_bool("show-docs") != Some(false) {
                            // If the `spec-url` has been configured, the user should
                            // provide the generated OpenAPI object with a derivation.
                            let path = config.get_str("rapidoc-route").unwrap_or("/rapidoc");
                            let rapidoc = if let Some(url) = config.get_str("spec-url") {
                                RapiDoc::new(url)
                            } else {
                                RapiDoc::with_openapi("/api-docs/openapi.json", openapi)
                            };
                            if let Some(custom_html) = config.get_str("custom-html") {
                                let custom_html_file = Self::parse_path(custom_html);
                                if let Ok(html) = fs::read_to_string(custom_html_file) {
                                    app = app.merge(rapidoc.custom_html(html).path(path));
                                } else {
                                    app = app.merge(rapidoc.path(path));
                                }
                            } else {
                                app = app.merge(rapidoc.path(path));
                            }
                            tracing::info!("RapiDoc router `{path}` is registered for `{addr}`");
                        }
                    } else {
                        let rapidoc = RapiDoc::with_openapi("/api-docs/openapi.json", openapi)
                            .path("/rapidoc");
                        app = app.merge(rapidoc);
                        tracing::info!("RapiDoc router `/rapidoc` is registered for `{addr}`");
                    }
                }

                app = app.layer(
                    ServiceBuilder::new()
                        .layer(SetResponseHeaderLayer::if_not_present(
                            HeaderName::from_static("connection"),
                            HeaderValue::from_static("keep-alive"),
                        ))
                        .layer(SetResponseHeaderLayer::if_not_present(
                            HeaderName::from_static("keep-alive"),
                            HeaderValue::from_str(&format!("timeout={keep_alive_timeout}"))
                                .expect("fail to set the `keep-alive` header value"),
                        ))
                        .layer(DefaultBodyLimit::max(body_limit))
                        .layer(
                            CompressionLayer::new()
                                .gzip(true)
                                .compress_when(DefaultPredicate::new()),
                        )
                        .layer(DecompressionLayer::new().gzip(true))
                        .layer(LazyLock::force(&middleware::TRACING_MIDDLEWARE))
                        .layer(LazyLock::force(&middleware::CORS_MIDDLEWARE))
                        .layer(from_fn(middleware::request_context))
                        .layer(from_fn(middleware::extract_etag))
                        .layer(HandleErrorLayer::new(|err: BoxError| async move {
                            let status_code = if err.is::<Elapsed>() {
                                StatusCode::REQUEST_TIMEOUT
                            } else if err.is::<LengthLimitError>() {
                                StatusCode::PAYLOAD_TOO_LARGE
                            } else {
                                StatusCode::INTERNAL_SERVER_ERROR
                            };
                            let res = Response::new(status_code);
                            Ok::<AxumResponse, Infallible>(res.into())
                        }))
                        .layer(CatchPanicLayer::custom(
                            |err: Box<dyn Any + Send + 'static>| {
                                let details = if let Some(s) = err.downcast_ref::<String>() {
                                    Cow::Owned(s.to_owned())
                                } else if let Some(s) = err.downcast_ref::<&str>() {
                                    Cow::Borrowed(*s)
                                } else {
                                    Cow::Borrowed("Unknown panic message")
                                };
                                let mut res = Response::internal_server_error();
                                res.set_message(details);
                                crate::response::build_http_response(res)
                            },
                        ))
                        .layer(TimeoutLayer::new(request_timeout)),
                );
                Box::pin(async move {
                    let tcp_listener = TcpListener::bind(&addr)
                        .await
                        .unwrap_or_else(|err| panic!("fail to listen on {addr}: {err}"));
                    axum::serve(
                        tcp_listener,
                        app.into_make_service_with_connect_info::<SocketAddr>(),
                    )
                    .with_graceful_shutdown(Self::shutdown())
                    .await
                })
            });
            for result in futures::future::join_all(servers).await {
                if let Err(err) = result {
                    tracing::error!("axum server error: {err}");
                }
            }
        });
    }

    async fn shutdown() {
        let ctrl_c = async {
            if let Err(err) = signal::ctrl_c().await {
                tracing::error!("fail to install the `Ctrl+C` handler: {err}");
            }
            #[cfg(feature = "orm")]
            zino_orm::GlobalPool::close_all().await;
        };
        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("fail to install the terminate signal handler")
                .recv()
                .await;
            #[cfg(feature = "orm")]
            zino_orm::GlobalPool::close_all().await;
        };
        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();
        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        };
        tracing::warn!("signal received, starting graceful shutdown");
    }
}
