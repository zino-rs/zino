//! High level abstractions for the application.

use crate::{
    datetime::DateTime,
    error::Error,
    extend::{HeaderMapExt, JsonObjectExt, TomlTableExt},
    schedule::{AsyncCronJob, CronJob, Job, JobScheduler},
    state::State,
    trace::TraceContext,
    Map,
};
use reqwest::Response;
use serde::de::DeserializeOwned;
use std::{env, path::PathBuf, sync::LazyLock, thread};
use toml::value::Table;

mod metrics_exporter;
mod secret_key;
mod system_monitor;
mod tracing_subscriber;

pub(crate) mod http_client;

pub(crate) use secret_key::SECRET_KEY;

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Registers routes.
    fn register(self, routes: Vec<Self::Router>) -> Self;

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

    /// Gets the system’s information.
    fn sysinfo() -> Map {
        system_monitor::refresh_and_retrieve()
    }

    /// Returns a reference to the shared application state.
    #[inline]
    fn shared_state() -> &'static State {
        LazyLock::force(&SHARED_APP_STATE)
    }

    /// Returns the application env.
    #[inline]
    fn env() -> &'static str {
        Self::shared_state().env()
    }

    /// Returns a reference to the shared application config.
    #[inline]
    fn config() -> &'static Table {
        Self::shared_state().config()
    }

    /// Returns a reference to the shared application state data.
    #[inline]
    fn state_data() -> &'static Map {
        Self::shared_state().data()
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

    /// Returns the secret key for the application.
    /// It should have at least 64 bytes.
    #[inline]
    fn secret_key() -> &'static [u8] {
        SECRET_KEY.get().expect("failed to get the secret key")
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
    /// deserializes the response body as JSON.
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
        .expect("the `name` field should be specified")
});

/// App version.
pub(crate) static APP_VERSION: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("version")
        .expect("the `version` field should be specified")
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
    env::current_dir()
        .expect("the project directory does not exist or permissions are insufficient")
});

/// Shared app state.
static SHARED_APP_STATE: LazyLock<State> = LazyLock::new(|| {
    let mut state = State::default();
    let config = state.config();
    let app_name = config
        .get_str("name")
        .expect("the `name` field should be a str");
    let app_version = config
        .get_str("version")
        .expect("the `version` field should be a str");

    let mut data = Map::new();
    data.upsert("app.name", app_name);
    data.upsert("app.version", app_version);
    data.upsert("app.booted_at", DateTime::now());
    state.set_data(data);
    state
});
