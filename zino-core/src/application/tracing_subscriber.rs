use super::Application;
use crate::extension::TomlTableExt;
use std::{fs, io, path::Path, sync::OnceLock, time::Duration};
use tracing::Level;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
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
    let in_dev_mode = app_env.is_dev();
    let mut env_filter = if in_dev_mode {
        "info,zino=trace,zino_core=trace"
    } else {
        "info"
    };

    let mut log_dir = "logs";
    let mut log_rotation = "hourly";
    let mut log_rolling_period = Duration::from_secs(3600 * 24 * 90); // 90 days
    let mut display_target = true;
    let mut display_filename = false;
    let mut display_line_number = false;
    let mut display_thread_names = false;
    let mut display_span_list = false;
    if let Some(config) = APP::config().get_table("tracing") {
        if let Some(dir) = config.get_str("log-dir") {
            log_dir = dir;
        }
        if let Some(rotation) = config.get_str("log-rotation") {
            log_rotation = rotation;
        }
        if let Some(period) = config.get_duration("log-rolling-period") {
            log_rolling_period = period;
        }
        if let Some(filter) = config.get_str("filter") {
            env_filter = filter;
        }
        display_target = config.get_bool("display-target").unwrap_or(true);
        display_filename = config.get_bool("display-filename").unwrap_or(in_dev_mode);
        display_line_number = config
            .get_bool("display-line-number")
            .unwrap_or(in_dev_mode);
        display_thread_names = config.get_bool("display-thread-names").unwrap_or(false);
        display_span_list = config.get_bool("display-span-list").unwrap_or(false);
    }

    let log_dir = Path::new(log_dir);
    let rolling_file_dir = if log_dir.exists() {
        log_dir.to_path_buf()
    } else {
        let project_dir = APP::project_dir();
        let log_dir = project_dir.join("logs");
        if !log_dir.exists() {
            fs::create_dir(log_dir.as_path()).unwrap_or_else(|err| {
                let log_dir = log_dir.display();
                panic!("fail to create the log directory `{log_dir}`: {err}");
            });
        }
        log_dir
    };

    let rolling_period_minutes = log_rolling_period.as_secs().div_ceil(60);
    let (rotation, max_log_files) = match log_rotation {
        "minutely" => (Rotation::MINUTELY, rolling_period_minutes),
        "hourly" => (Rotation::HOURLY, rolling_period_minutes.div_ceil(60)),
        "daily" => (Rotation::DAILY, rolling_period_minutes.div_ceil(60 * 24)),
        _ => (Rotation::NEVER, 1),
    };

    let app_name = APP::name();
    let file_appender = RollingFileAppender::builder()
        .rotation(rotation)
        .filename_prefix(format!("{app_name}.{app_env}"))
        .filename_suffix("log")
        .max_log_files(max_log_files.try_into().unwrap_or(1))
        .build(rolling_file_dir)
        .expect("fail to initialize the rolling file appender");
    let (non_blocking_appender, worker_guard) = tracing_appender::non_blocking(file_appender);
    let stdout = if in_dev_mode {
        io::stdout.with_max_level(Level::DEBUG)
    } else {
        io::stdout.with_max_level(Level::WARN)
    };
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(display_target)
        .with_file(display_filename)
        .with_line_number(display_line_number)
        .with_thread_names(display_thread_names)
        .with_timer(local_offset_time)
        .with_writer(stdout.and(non_blocking_appender));
    let filter_layer = EnvFilter::builder()
        .with_default_directive(LevelFilter::WARN.into())
        .parse_lossy(env_filter);
    if in_dev_mode {
        let pretty_fmt_layer = fmt_layer.pretty();
        let subscriber = tracing_subscriber::registry()
            .with(filter_layer)
            .with(pretty_fmt_layer);
        tracing::subscriber::set_global_default(subscriber)
            .expect("fail to set the default subscriber with a `Pretty` formatter");
    } else {
        let json_fmt_layer = fmt_layer
            .json()
            .with_current_span(true)
            .with_span_list(display_span_list);
        let subscriber = tracing_subscriber::registry()
            .with(filter_layer)
            .with(json_fmt_layer);
        tracing::subscriber::set_global_default(subscriber)
            .expect("fail to set the default subscriber with a `Json` formatter");
    };
    TRACING_APPENDER_GUARD
        .set(worker_guard)
        .expect("fail to set the worker guard for the tracing appender");
}

/// Tracing appender guard.
static TRACING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
