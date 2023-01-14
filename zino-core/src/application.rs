//! Application trait.

use crate::{
    schedule::{AsyncCronJob, CronJob, Job, JobScheduler},
    state::State,
    Map,
};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use metrics_exporter_tcp::TcpBuilder;
use std::{
    collections::HashMap,
    env, fs, io,
    net::IpAddr,
    path::{Path, PathBuf},
    sync::{LazyLock, OnceLock},
    thread,
    time::Duration,
};
use toml::value::Table;
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::fmt::{time, writer::MakeWriterExt};

/// Application.
pub trait Application {
    /// Router.
    type Router;

    /// Creates a new application.
    fn new() -> Self;

    /// Returns a reference to the shared application state.
    fn shared_state() -> &'static State;

    /// Registers routes.
    fn register(self, routes: HashMap<&'static str, Self::Router>) -> Self;

    /// Runs the application.
    fn run(self, async_jobs: HashMap<&'static str, AsyncCronJob>);

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
            .get("name")
            .and_then(|t| t.as_str())
            .expect("the `name` field should be specified")
    }

    /// Returns the application version.
    #[inline]
    fn version() -> &'static str {
        Self::config()
            .get("version")
            .and_then(|t| t.as_str())
            .expect("the `version` field should be specified")
    }

    /// Returns the project directory for the application.
    #[inline]
    fn project_dir() -> &'static PathBuf {
        LazyLock::force(&PROJECT_DIR)
    }

    /// Initializes the application. It setups the tracing subscriber and the metrics exporter.
    fn init() {
        if TRACING_APPENDER_GUARD.get().is_some() {
            tracing::warn!("the tracing subscriber has already been initialized");
            return;
        }

        let app_env = Self::env();
        let mut log_dir = "./log";
        let mut env_filter = if app_env == "dev" {
            "info,sqlx=trace,zino=trace,zino_core=trace"
        } else {
            "info,sqlx=warn"
        };
        let mut display_target = true;
        let mut display_filename = false;
        let mut display_line_number = false;
        let mut display_thread_names = false;
        let mut display_span_list = false;
        let display_current_span = true;

        let config = Self::config();
        if let Some(tracing) = config.get("tracing").and_then(|t| t.as_table()) {
            if let Some(dir) = tracing.get("log-dir").and_then(|t| t.as_str()) {
                log_dir = dir;
            }
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

        let app_name = Self::name();
        let log_dir = Path::new(log_dir);
        let rolling_file_dir = if log_dir.exists() {
            log_dir.to_path_buf()
        } else {
            let project_dir = Self::project_dir();
            let log_dir = project_dir.join("./log");
            if !log_dir.exists() {
                fs::create_dir(log_dir.as_path()).unwrap_or_else(|err| {
                    let log_dir = log_dir.to_string_lossy();
                    panic!("failed to create the log directory `{log_dir}`: {err}");
                });
            }
            log_dir
        };
        let file_appender = rolling::hourly(rolling_file_dir, format!("{app_name}.{app_env}"));
        let (non_blocking_appender, worker_guard) = tracing_appender::non_blocking(file_appender);
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
            .with_writer(stderr.and(non_blocking_appender))
            .init();
        TRACING_APPENDER_GUARD
            .set(worker_guard)
            .expect("failed to set the worker guard for the tracing appender");

        if let Some(metrics) = config.get("metrics").and_then(|t| t.as_table()) {
            let exporter = metrics
                .get("exporter")
                .and_then(|t| t.as_str())
                .unwrap_or_default();
            if exporter == "prometheus" {
                let mut builder = match metrics.get("push-gateway").and_then(|t| t.as_str()) {
                    Some(endpoint) => {
                        let interval = metrics
                            .get("interval")
                            .and_then(|t| t.as_integer().and_then(|i| i.try_into().ok()))
                            .unwrap_or(60);
                        PrometheusBuilder::new()
                            .with_push_gateway(endpoint, Duration::from_secs(interval))
                            .expect("failed to configure the exporter to run in push gateway mode")
                    }
                    None => {
                        let host = config
                            .get("host")
                            .and_then(|t| t.as_str())
                            .unwrap_or("127.0.0.1");
                        let port = config
                            .get("port")
                            .and_then(|t| t.as_integer())
                            .and_then(|t| u16::try_from(t).ok())
                            .unwrap_or(9000);
                        let host_addr = host
                            .parse::<IpAddr>()
                            .unwrap_or_else(|err| panic!("invalid host address `{host}`: {err}"));
                        PrometheusBuilder::new().with_http_listener((host_addr, port))
                    }
                };
                if let Some(quantiles) = config.get("quantiles").and_then(|t| t.as_array()) {
                    let quantiles = quantiles
                        .iter()
                        .filter_map(|q| q.as_float())
                        .collect::<Vec<_>>();
                    builder = builder
                        .set_quantiles(&quantiles)
                        .expect("invalid quantiles to render histograms");
                }
                if let Some(buckets) = config.get("buckets").and_then(|t| t.as_table()) {
                    for (key, value) in buckets {
                        let matcher = if key.starts_with('^') {
                            Matcher::Prefix(key.to_string())
                        } else if key.ends_with('$') {
                            Matcher::Suffix(key.to_string())
                        } else {
                            Matcher::Full(key.to_string())
                        };
                        let values = value
                            .as_array()
                            .expect("buckets should be an array of floats")
                            .iter()
                            .filter_map(|v| v.as_float())
                            .collect::<Vec<_>>();
                        builder = builder
                            .set_buckets_for_metric(matcher, &values)
                            .expect("invalid buckets to render histograms");
                    }
                }
                if let Some(labels) = config.get("global-labels").and_then(|t| t.as_table()) {
                    for (key, value) in labels {
                        builder = builder.add_global_label(key, value.to_string());
                    }
                }
                if let Some(addresses) = config.get("allowed-addresses").and_then(|t| t.as_array())
                {
                    for addr in addresses {
                        builder = builder
                            .add_allowed_address(addr.as_str().unwrap_or_default())
                            .unwrap_or_else(|err| panic!("invalid IP address `{addr}`: {err}"));
                    }
                }
                builder
                    .install()
                    .expect("failed to install Prometheus exporter");
            } else if exporter == "tcp" {
                let host = config
                    .get("host")
                    .and_then(|t| t.as_str())
                    .unwrap_or("127.0.0.1");
                let port = config
                    .get("port")
                    .and_then(|t| t.as_integer())
                    .and_then(|t| u16::try_from(t).ok())
                    .unwrap_or(9000);
                let buffer_size = config
                    .get("buffer_size")
                    .and_then(|t| t.as_integer())
                    .and_then(|t| usize::try_from(t).ok())
                    .unwrap_or(1024);
                let host_addr = host
                    .parse::<IpAddr>()
                    .unwrap_or_else(|err| panic!("invalid host address `{host}`: {err}"));
                TcpBuilder::new()
                    .listen_address((host_addr, port))
                    .buffer_size(Some(buffer_size))
                    .install()
                    .expect("failed to install TCP exporter");
            }
        }
    }
}

/// Project directory.
static PROJECT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::current_dir()
        .expect("the project directory does not exist or permissions are insufficient")
});

/// Tracing appender guard.
static TRACING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
