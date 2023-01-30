//! Application utilities.

use crate::{
    datetime::DateTime,
    extend::{JsonObjectExt, TomlTableExt},
    schedule::{AsyncCronJob, CronJob, Job, JobScheduler},
    state::State,
    trace::TraceContext,
    BoxError, Map,
};
use reqwest::{IntoUrl, Response};
use std::{collections::HashMap, env, path::PathBuf, sync::LazyLock, thread};
use toml::value::Table;

mod metrics_exporter;
mod system_monitor;
mod tracing_subscriber;

pub(crate) mod http_client;

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Registers routes.
    fn register(self, routes: Vec<Self::Router>) -> Self;

    /// Runs the application.
    fn run(self, async_jobs: HashMap<&'static str, AsyncCronJob>);

    /// Boots the application. It also setups the tracing subscriber, the metrics exporter
    /// and a global HTTP client.
    fn boot() -> Self
    where
        Self: Default,
    {
        tracing_subscriber::init::<Self>();
        metrics_exporter::init::<Self>();
        http_client::init::<Self>();
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
        Self::config()
            .get_str("name")
            .expect("the `name` field should be specified")
    }

    /// Returns the application version.
    #[inline]
    fn version() -> &'static str {
        Self::config()
            .get_str("version")
            .expect("the `version` field should be specified")
    }

    /// Returns the project directory for the application.
    #[inline]
    fn project_dir() -> &'static PathBuf {
        LazyLock::force(&PROJECT_DIR)
    }

    /// Spawns a new thread to run cron jobs.
    fn spawn(self, jobs: HashMap<&'static str, CronJob>) -> Self
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
    async fn fetch(
        resource: impl IntoUrl,
        options: impl Into<Option<Map>>,
    ) -> Result<Response, BoxError> {
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
            .map_err(BoxError::from)
    }
}

/// Project directory.
static PROJECT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
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
    data.upsert("app.start_at", DateTime::now());
    state.set_data(data);
    state
});
