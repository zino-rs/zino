use super::Application;
use crate::extension::TomlTableExt;
use std::{fs, io, sync::OnceLock, time::Duration};
use tracing::Level;
use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    filter::LevelFilter,
    fmt::{time::OffsetTime, writer::MakeWriterExt},
    layer::SubscriberExt,
};

#[cfg(feature = "sentry")]
use sentry_tracing::EventFilter;

#[cfg(feature = "env-filter")]
use tracing_subscriber::filter::EnvFilter;

/// Returns the default log directory for the application.
fn default_log_dir<APP: Application + ?Sized>() -> String {
    if cfg!(not(debug_assertions)) && APP::APP_TYPE.is_desktop() && APP::env().is_prod() {
        let app_name = APP::name();
        if cfg!(target_os = "windows") {
            format!("~/AppData/Roaming/{app_name}/Logs")
        } else if cfg!(target_os = "macos") {
            format!("~/Library/Logs/{app_name}")
        } else {
            format!("~/.local/share/{app_name}/logs")
        }
    } else {
        "logs".to_owned()
    }
}

/// Initializes the tracing subscriber.
pub(super) fn init<APP: Application + ?Sized>() {
    if TRACING_APPENDER_GUARD.get().is_some() {
        tracing::warn!("tracing subscriber has already been initialized");
        return;
    }

    // Converts log records to tracing events
    #[cfg(feature = "tracing-log")]
    tracing_log::LogTracer::init().expect("fail to initialize the log tracer");

    // Initializes `OffsetTime` before forking threads
    let local_offset_time = OffsetTime::local_rfc_3339().expect("could not get local offset");

    // Initializes the sentry client
    #[cfg(feature = "sentry")]
    super::sentry_client::init::<APP>();

    let app_env = APP::env();
    let in_dev_mode = app_env.is_dev();
    let mut event_format = if in_dev_mode { "pretty" } else { "json" };
    let mut level_filter = if in_dev_mode {
        LevelFilter::INFO
    } else {
        LevelFilter::WARN
    };
    let mut stdout_max_level = if in_dev_mode {
        Level::DEBUG
    } else {
        Level::WARN
    };
    #[cfg(feature = "env-filter")]
    let mut env_filter = if in_dev_mode {
        "info,zino=trace,zino_core=trace"
    } else {
        "warn,zino=info,zino_core=info"
    };

    let mut log_dir = default_log_dir::<APP>();
    let mut log_rotation = "hourly";
    let mut log_rolling_period = Duration::from_secs(3600 * 24 * 90); // 90 days
    let mut ansi_terminal = true;
    let mut display_target = true;
    let mut display_filename = false;
    let mut display_line_number = false;
    let mut display_thread_ids = false;
    let mut display_thread_names = false;
    let mut display_span_list = false;
    let mut flatten_event = false;
    if let Some(config) = APP::config().get_table("tracing") {
        if let Some(dir) = config.get_str("log-dir") {
            log_dir = dir.to_owned();
        }
        if let Some(rotation) = config.get_str("log-rotation") {
            log_rotation = rotation;
        }
        if let Some(period) = config.get_duration("log-rolling-period") {
            log_rolling_period = period;
        }
        if let Some(format) = config.get_str("format") {
            event_format = format;
        }
        if let Some(level) = config.get_str("level") {
            stdout_max_level = level.parse().expect("fail to parse the level");
            level_filter = level.parse().expect("fail to parse the level filter");
        }
        #[cfg(feature = "env-filter")]
        if let Some(filter) = config.get_str("filter") {
            env_filter = filter;
        }
        ansi_terminal = config.get_bool("ansi").unwrap_or(true);
        display_target = config.get_bool("display-target").unwrap_or(true);
        display_filename = config.get_bool("display-filename").unwrap_or(in_dev_mode);
        display_line_number = config
            .get_bool("display-line-number")
            .unwrap_or(in_dev_mode);
        display_thread_ids = config.get_bool("display-thread-ids").unwrap_or(false);
        display_thread_names = config.get_bool("display-thread-names").unwrap_or(false);
        display_span_list = config.get_bool("display-span-list").unwrap_or(false);
        flatten_event = config.get_bool("flatten-event").unwrap_or(false);
    }

    let log_dir = APP::parse_path(&log_dir);
    if !log_dir.exists() {
        fs::create_dir_all(&log_dir).unwrap_or_else(|err| {
            let log_dir = log_dir.display();
            panic!("fail to create the log directory `{log_dir}`: {err}");
        });
    }

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
        .build(log_dir)
        .expect("fail to initialize the rolling file appender");
    let (non_blocking_appender, worker_guard) = tracing_appender::non_blocking(file_appender);

    // Format layer
    let stdout = io::stdout.with_max_level(stdout_max_level);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(ansi_terminal)
        .with_target(display_target)
        .with_file(display_filename)
        .with_line_number(display_line_number)
        .with_thread_ids(display_thread_ids)
        .with_thread_names(display_thread_names)
        .with_timer(local_offset_time)
        .with_writer(stdout.and(non_blocking_appender));

    // Optional layers
    #[cfg(feature = "env-filter")]
    let env_filter_layer = EnvFilter::builder()
        .with_default_directive(level_filter.into())
        .parse(env_filter)
        .expect("fail to parse the env filter");
    #[cfg(feature = "sentry")]
    let sentry_layer = sentry_tracing::layer()
        .enable_span_attributes()
        .event_filter(|md| match *md.level() {
            Level::ERROR => EventFilter::Exception,
            Level::WARN | Level::INFO => EventFilter::Breadcrumb,
            _ => EventFilter::Ignore,
        });

    let subscriber = tracing_subscriber::registry();
    #[cfg(feature = "env-filter")]
    let subscriber = subscriber.with(env_filter_layer);
    #[cfg(not(feature = "env-filter"))]
    let subscriber = subscriber.with(level_filter);
    #[cfg(feature = "sentry")]
    let subscriber = subscriber.with(sentry_layer);
    match event_format {
        "compact" => {
            let compact_fmt_layer = fmt_layer.compact();
            let subscriber = subscriber.with(compact_fmt_layer);
            if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
                tracing::warn!(
                    "fail to set the default subscriber with a `Compact` formatter: {err}"
                );
            }
        }
        "json" => {
            let json_fmt_layer = fmt_layer
                .json()
                .flatten_event(flatten_event)
                .with_current_span(true)
                .with_span_list(display_span_list);
            let subscriber = subscriber.with(json_fmt_layer);
            if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
                tracing::warn!("fail to set the default subscriber with a `Json` formatter: {err}");
            }
        }
        "pretty" => {
            let pretty_fmt_layer = fmt_layer.pretty();
            let subscriber = subscriber.with(pretty_fmt_layer);
            if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
                tracing::warn!(
                    "fail to set the default subscriber with a `Pretty` formatter: {err}"
                );
            }
        }
        _ => {
            let subscriber = subscriber.with(fmt_layer);
            if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
                tracing::warn!("fail to set the default subscriber with a `Full` formatter: {err}");
            }
        }
    }
    TRACING_APPENDER_GUARD
        .set(worker_guard)
        .expect("fail to set the worker guard for the tracing appender");
}

/// Tracing appender guard.
static TRACING_APPENDER_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
