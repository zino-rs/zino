use axum::{response::Html, routing::get, Router};
use dioxus::prelude::*;
use dioxus_router::{components::Router, routable::Routable};
use dioxus_ssr::renderer::Renderer;
use std::{fmt::Display, marker::PhantomData, net::SocketAddr, str::FromStr, time::Duration};
use tokio::{net::TcpListener, runtime::Builder, signal};
use zino_core::{
    application::{Application, Plugin},
    schedule::AsyncScheduler,
};

/// Server-side rendering for the Dioxus VirtualDom.
#[derive(Default)]
pub struct DioxusSsr<R> {
    /// Custom plugins.
    custom_plugins: Vec<Plugin>,
    /// Phantom type of Dioxus router.
    phantom: PhantomData<R>,
}

impl<R> DioxusSsr<R>
where
    R: Routable,
    <R as FromStr>::Err: Display,
{
    /// Renders the app root.
    fn app_root() -> Element {
        rsx! {
            head {
                meta { charset: "utf-8" }
                meta { name: "viewport", content: "width=device-width, initial-scale=1" }
            }
            body {
                Router::<R> {}
            }
        }
    }

    /// Returns the rendered HTML.
    async fn app_endpoint() -> Html<String> {
        let mut vdom = VirtualDom::new(Self::app_root);
        vdom.rebuild_in_place();

        let mut renderer = Renderer::new();
        renderer.pretty = true;
        renderer.newline = true;
        renderer.pre_render = false;
        Html(renderer.render(&vdom))
    }
}

impl<R> Application for DioxusSsr<R>
where
    R: Routable,
    <R as FromStr>::Err: Display,
{
    type Routes = R;

    fn register(self, _routes: Self::Routes) -> Self {
        self
    }

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
            .expect("fail to build Tokio runtime for `DioxusSsr`");
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
                    tokio::time::sleep(scheduler.time_till_next_job()).await;
                }
            });
        }

        runtime.block_on(async {
            let app_state = Self::shared_state();
            let app_name = Self::name();
            let app_version = Self::version();
            let listeners = app_state.listeners();
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

                let app = Router::new().route_service("/", get(Self::app_endpoint));
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
                    tracing::error!("SSR server error: {err}");
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
            zino_core::orm::GlobalPool::close_all().await;
        };
        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("fail to install the terminate signal handler")
                .recv()
                .await;
            #[cfg(feature = "orm")]
            zino_core::orm::GlobalPool::close_all().await;
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
