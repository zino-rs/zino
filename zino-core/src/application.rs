use crate::{AsyncCronJob, CronJob, Job, JobScheduler, Map, State};
use std::{
    collections::HashMap,
    env, io,
    path::PathBuf,
    sync::{LazyLock, OnceLock},
    thread,
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

    /// Initializes the tracing subscriber.
    fn init_tracing_subscriber() {
        if TRACING_APPENDER_GUARD.get().is_some() {
            tracing::warn!("the tracing subscriber has already been initialized");
            return;
        }

        let app_env = Self::env();
        let is_dev = app_env == "dev";
        let mut env_filter = if is_dev {
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

        let subscriber = tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .with_target(display_target)
            .with_file(display_filename)
            .with_line_number(display_line_number)
            .with_thread_names(display_thread_names)
            .with_span_list(display_span_list)
            .with_current_span(display_current_span)
            .with_timer(time::LocalTime::rfc_3339());

        let app_name = Self::name();
        let project_dir = Self::project_dir();
        let log_dir = project_dir.join("./log");
        let rolling_file_dir = if log_dir.exists() {
            log_dir
        } else {
            project_dir.join("../log")
        };
        let file_appender = rolling::hourly(rolling_file_dir, format!("{app_name}.{app_env}"));
        let (non_blocking_appender, worker_guard) = tracing_appender::non_blocking(file_appender);
        if is_dev {
            let stdout = io::stdout.with_max_level(Level::WARN);
            subscriber
                .with_writer(stdout.and(non_blocking_appender))
                .init();
        } else {
            subscriber.with_writer(non_blocking_appender).init();
        }
        TRACING_APPENDER_GUARD
            .set(worker_guard)
            .expect("fail to set the worker guard for the tracing appender");
    }
}

/// Project directory.
static PROJECT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::current_dir()
        .expect("the project directory does not exist or permissions are insufficient")
});

/// Tracing appender guard.
static TRACING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
