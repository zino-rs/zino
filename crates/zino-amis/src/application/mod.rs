//! UI generator for amis.

use hyper::{Request, Uri, body::Incoming, server::conn::http1, service};
use hyper_staticfile::{AcceptEncoding, Static};
use hyper_util::rt::TokioIo;
use std::{fs, time::Duration};
use tokio::{net::TcpListener, runtime::Builder};
use zino_core::{
    application::{AppType, Application, StaticRecord},
    extension::TomlTableExt,
    schedule::AsyncScheduler,
};

/// UI generator for amis.
#[derive(Default)]
pub struct Amis {
    /// Routes.
    routes: StaticRecord<Uri>,
}

impl Application for Amis {
    type Routes = phf::Map<&'static str, &'static str>;

    const APP_TYPE: AppType = AppType::Web;

    fn register(mut self, routes: Self::Routes) -> Self {
        let mut records = StaticRecord::with_capacity(routes.len());
        for (&name, uri) in &routes {
            if let Ok(uri) = uri.parse() {
                records.add(name, uri);
            }
        }
        self.routes = records;
        self
    }

    #[inline]
    fn run_with<T: AsyncScheduler + Send + 'static>(self, mut scheduler: T) {
        let runtime = Builder::new_multi_thread()
            .thread_keep_alive(Duration::from_secs(60))
            .thread_stack_size(2 * 1024 * 1024)
            .global_queue_interval(61)
            .enable_all()
            .build()
            .expect("fail to build Tokio runtime for `DesktopUi` generator");
        runtime.block_on(async {
            Self::load().await;
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
            let app_state = Self::shared_state();
            let app_env = Self::env();
            let app_name = Self::name();
            let app_version = Self::version();
            let socket_addrs = app_state
                .listeners()
                .into_iter()
                .map(|(server_tag, addr)| {
                    tracing::warn!(
                        server_tag = server_tag.as_str(),
                        app_env = app_env.as_str(),
                        app_name,
                        app_version,
                        zino_version = env!("CARGO_PKG_VERSION"),
                        "listen on `{addr}`",
                    );
                    addr
                })
                .collect::<Vec<_>>();
            let listener = TcpListener::bind(socket_addrs.as_slice())
                .await
                .expect("fail to create a TCP listener");

            // Server config
            let mut public_dir = "public";
            let mut amis_dir = "public/amis";
            if let Some(config) = app_state.get_config("server") {
                if let Some(dir) = config.get_str("public-dir") {
                    public_dir = dir;
                }
                if let Some(dir) = config.get_str("amis-dir") {
                    amis_dir = dir;
                }
            }

            // Generate amis schemas.
            let config_dir = Self::config_dir().join("amis");
            let output_dir = Self::parse_path(amis_dir);
            if !output_dir.exists() {
                if let Err(err) = fs::create_dir(&output_dir) {
                    tracing::error!("fail to create amis output dir: {err}");
                }
            }
            if let Err(err) = crate::amis::compile(&config_dir, &output_dir) {
                tracing::error!("fail to generate amis schemas: {err}");
            }

            let routes = self.routes.leak();
            let public_dir = Self::parse_path(public_dir);
            let mut static_files = Static::new(public_dir);
            static_files.allowed_encodings(AcceptEncoding::all());
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        let static_files = static_files.clone();
                        tokio::spawn(async move {
                            let serve_file = service::service_fn(|mut req: Request<Incoming>| {
                                let static_files = static_files.clone();
                                let path = req.uri().path();
                                if !path.contains('.') {
                                    let name = path.trim_end_matches('/');
                                    if let Some(uri) =
                                        routes.iter().find_map(|(k, v)| (*k == name).then_some(v))
                                    {
                                        *req.uri_mut() = uri.to_owned();
                                    }
                                }
                                async move { static_files.clone().serve(req).await }
                            });
                            if let Err(err) = http1::Builder::new()
                                .serve_connection(TokioIo::new(stream), serve_file)
                                .await
                            {
                                tracing::error!("hyper server connection error: {err}");
                            }
                        });
                    }
                    Err(err) => tracing::error!("hyper server error: {err}"),
                }
            }
        });
    }
}
