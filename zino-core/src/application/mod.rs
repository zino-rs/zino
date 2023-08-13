//! High level abstractions for the application.

use crate::{
    datetime::DateTime,
    error::Error,
    extension::{HeaderMapExt, JsonObjectExt, TomlTableExt},
    openapi,
    schedule::{AsyncCronJob, CronJob, Job, JobScheduler},
    state::State,
    trace::TraceContext,
    Map,
};
use reqwest::Response;
use serde::de::DeserializeOwned;
use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::LazyLock,
    thread,
};
use toml::value::Table;
use utoipa::openapi::{Info, OpenApi, OpenApiBuilder};

mod metrics_exporter;
mod secret_key;
mod system_monitor;
mod tracing_subscriber;

pub(crate) mod http_client;

pub(crate) use secret_key::SECRET_KEY;

/// Application.
pub trait Application {
    /// Routes.
    type Routes;

    /// Registers routes.
    fn register(self, routes: Self::Routes) -> Self;

    /// Runs the application.
    fn run(self, async_jobs: Vec<(&'static str, AsyncCronJob)>);

    /// Boots the application. It also setups the default secret key,
    /// the tracing subscriber, the metrics exporter and a global HTTP client.
    fn boot() -> Self
    where
        Self: Default,
    {
        secret_key::init::<Self>();
        tracing_subscriber::init::<Self>();
        metrics_exporter::init::<Self>();
        http_client::init::<Self>();

        #[cfg(feature = "view")]
        {
            crate::view::init::<Self>();
        }

        Self::default()
    }

    /// Boots the application with a custom initialization.
    fn boot_with<F>(init: F) -> Self
    where
        Self: Default,
        F: FnOnce(&'static State<Map>),
    {
        init(Self::shared_state());
        Self::boot()
    }

    /// Initializes the directories to ensure that they are ready for use,
    /// then boots the application.
    fn init_dirs(dirs: &[&'static str]) -> Self
    where
        Self: Default,
    {
        let project_dir = Self::project_dir();
        for dir in dirs {
            let path = if dir.starts_with('/') {
                PathBuf::from(dir)
            } else {
                project_dir.join(dir)
            };
            if !path.exists() && let Err(err) = fs::create_dir_all(&path) {
                let path = path.to_string_lossy();
                tracing::error!("fail to create the directory {}: {}", path, err);
            }
        }
        Self::boot()
    }

    /// Gets the system’s information.
    #[inline]
    fn sysinfo() -> Map {
        system_monitor::refresh_and_retrieve()
    }

    /// Gets the [OpenAPI](https://spec.openapis.org/oas/latest.html) document.
    #[inline]
    fn openapi() -> OpenApi {
        OpenApiBuilder::new()
            .info(Info::new(Self::name(), Self::version()))
            .paths(openapi::default_paths())
            .components(Some(openapi::default_components()))
            .tags(Some(openapi::default_tags()))
            .build()
    }

    /// Returns a reference to the shared application state.
    #[inline]
    fn shared_state() -> &'static State<Map> {
        LazyLock::force(&SHARED_APP_STATE)
    }

    /// Returns the application env.
    #[inline]
    fn env() -> &'static str {
        SHARED_APP_STATE.env()
    }

    /// Returns a reference to the shared application config.
    #[inline]
    fn config() -> &'static Table {
        SHARED_APP_STATE.config()
    }

    /// Returns a reference to the shared application state data.
    #[inline]
    fn state_data() -> &'static Map {
        SHARED_APP_STATE.data()
    }

    /// Returns the application name.
    #[inline]
    fn name() -> &'static str {
        APP_NMAE.as_ref()
    }

    /// Returns the application version.
    #[inline]
    fn version() -> &'static str {
        APP_VERSION.as_ref()
    }

    /// Returns the domain for the application.
    #[inline]
    fn domain() -> &'static str {
        APP_DOMAIN.as_ref()
    }

    /// Returns the project directory for the application.
    #[inline]
    fn project_dir() -> &'static PathBuf {
        LazyLock::force(&PROJECT_DIR)
    }

    /// Returns the path relative to the project directory.
    #[inline]
    fn relative_path<P: AsRef<Path>>(path: P) -> PathBuf {
        PROJECT_DIR.join(path)
    }

    /// Returns the secret key for the application.
    /// It should have at least 64 bytes.
    ///
    /// # Note
    ///
    /// This should only be used for internal services. Do not expose it to external users.
    #[inline]
    fn secret_key() -> &'static [u8] {
        SECRET_KEY.get().expect("fail to get the secret key")
    }

    /// Spawns a new thread to run cron jobs.
    fn spawn(self, jobs: Vec<(&'static str, CronJob)>) -> Self
    where
        Self: Sized,
    {
        let mut scheduler = JobScheduler::new();
        for (cron_expr, exec) in jobs {
            scheduler.add(Job::new(cron_expr, exec));
        }
        thread::spawn(move || loop {
            scheduler.tick();
            thread::sleep(scheduler.time_till_next_job());
        });
        self
    }

    /// Makes an HTTP request to the provided resource
    /// using [`reqwest`](https://crates.io/crates/reqwest).
    async fn fetch(resource: &str, options: Option<&Map>) -> Result<Response, Error> {
        let mut trace_context = TraceContext::new();
        let span_id = trace_context.span_id();
        trace_context
            .trace_state_mut()
            .push("zino", format!("{span_id:x}"));
        http_client::request_builder(resource, options)?
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await
            .map_err(Error::from)
    }

    /// Makes an HTTP request to the provided resource and
    /// deserializes the response body via JSON.
    async fn fetch_json<T: DeserializeOwned>(
        resource: &str,
        options: Option<&Map>,
    ) -> Result<T, Error> {
        let response = Self::fetch(resource, options).await?.error_for_status()?;
        let data = if response.headers().has_json_content_type() {
            response.json().await?
        } else {
            let text = response.text().await?;
            serde_json::from_str(&text)?
        };
        Ok(data)
    }
}

/// App name.
pub(crate) static APP_NMAE: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("name")
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_NAME")
                .expect("fail to get the environment variable `CARGO_PKG_NAME`")
                .leak()
        })
});

/// App version.
pub(crate) static APP_VERSION: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("version")
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_VERSION")
                .expect("fail to get the environment variable `CARGO_PKG_VERSION`")
                .leak()
        })
});

/// Domain.
pub(crate) static APP_DOMAIN: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("domain")
        .unwrap_or("localhost")
});

/// Project directory.
pub(crate) static PROJECT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("CARGO_MANIFEST_DIR")
        .expect("fail to get the environment variable `CARGO_MANIFEST_DIR`")
        .into()
});

/// Shared app state.
static SHARED_APP_STATE: LazyLock<State<Map>> = LazyLock::new(|| {
    let mut state = State::default();
    state.load_config();

    let config = state.config();
    let app_name = config
        .get_str("name")
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_NAME")
                .expect("fail to get the environment variable `CARGO_PKG_NAME`")
        });
    let app_version = config
        .get_str("version")
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_VERSION")
                .expect("fail to get the environment variable `CARGO_PKG_VERSION`")
        });

    let mut data = Map::new();
    data.upsert("app.name", app_name);
    data.upsert("app.version", app_version);
    data.upsert("app.booted_at", DateTime::now());
    state.set_data(data);
    state
});
