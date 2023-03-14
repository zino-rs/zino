use super::Application;
use crate::extend::TomlTableExt;
use std::{fs, io, path::Path, sync::OnceLock};
use tracing::Level;
use tracing_appender::{non_blocking::WorkerGuard, rolling};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt::{time::OffsetTime, writer::MakeWriterExt},
    layer::SubscriberExt,
};

/// Initializes the tracing subscriber.
pub(super) fn init<APP: Application + ?Sized>() {
    if TRACING_APPENDER_GUARD.get().is_some() {
        tracing::warn!("the tracing subscriber has already been initialized");
        return;
    }

    // Initialize `OffsetTime` before forking threads.
    let local_offset_time = OffsetTime::local_rfc_3339().expect("could not get local offset");

    let app_env = APP::env();
    let mut env_filter = if app_env == "dev" {
        "info,sqlx=trace,zino=trace,zino_core=trace"
    } else {
        "info,sqlx=warn"
    };

    let mut log_dir = "logs";
    let mut display_target = true;
    let mut display_filename = false;
    let mut display_line_number = false;
    let mut display_thread_names = false;
    let mut display_span_list = false;
    if let Some(tracing) = APP::config().get_table("tracing") {
        if let Some(dir) = tracing.get_str("log-dir") {
            log_dir = dir;
        }
        if let Some(filter) = tracing.get_str("filter") {
            env_filter = filter;
        }
        display_target = tracing.get_bool("display-target").unwrap_or(true);
        display_filename = tracing.get_bool("display-filename").unwrap_or(false);
        display_line_number = tracing.get_bool("display-line-number").unwrap_or(false);
        display_thread_names = tracing.get_bool("display-thread-names").unwrap_or(false);
        display_span_list = tracing.get_bool("display-span-list").unwrap_or(false);
    }

    let log_dir = Path::new(log_dir);
    let rolling_file_dir = if log_dir.exists() {
        log_dir.to_path_buf()
    } else {
        let project_dir = APP::project_dir();
        let log_dir = project_dir.join("logs");
        if !log_dir.exists() {
            fs::create_dir(log_dir.as_path()).unwrap_or_else(|err| {
                let log_dir = log_dir.to_string_lossy();
                panic!("fail to create the log directory `{log_dir}`: {err}");
            });
        }
        log_dir
    };

    let app_name = APP::name();
    let file_appender = rolling::hourly(rolling_file_dir, format!("{app_name}.{app_env}"));
    let (non_blocking_appender, worker_guard) = tracing_appender::non_blocking(file_appender);
    let stderr = io::stderr.with_max_level(Level::WARN);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(display_target)
        .with_file(display_filename)
        .with_line_number(display_line_number)
        .with_thread_names(display_thread_names)
        .with_timer(local_offset_time)
        .with_writer(stderr.and(non_blocking_appender))
        .json()
        .with_current_span(true)
        .with_span_list(display_span_list);
    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .parse_lossy(env_filter);
    let subscriber = tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer);
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
    TRACING_APPENDER_GUARD
        .set(worker_guard)
        .expect("fail to set the worker guard for the tracing appender");
}

/// Tracing appender guard.
static TRACING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
