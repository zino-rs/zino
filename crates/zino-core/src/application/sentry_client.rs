use super::Application;
use crate::extension::TomlTableExt;
use sentry::{ClientInitGuard, ClientOptions, SessionMode};
use std::sync::OnceLock;

/// Initializes the sentry client.
pub(super) fn init<APP: Application + ?Sized>() {
    if SENTRY_CLIENT_GUARD.get().is_some() {
        tracing::warn!("sentry client has already been initialized");
        return;
    }

    let app_env = APP::env();
    let in_dev_mode = app_env.is_dev();
    let mut client_options = ClientOptions {
        debug: in_dev_mode,
        environment: Some(app_env.as_str().into()),
        traces_sample_rate: 1.0,
        ..Default::default()
    };
    if let Some(config) = APP::config().get_table("sentry") {
        if let Some(dsn) = config.get_str("dsn") {
            client_options.dsn = dsn.parse().ok();
        }
        if let Some(debug) = config.get_bool("debug") {
            client_options.debug = debug;
        }
        if let Some(release) = config.get_str("release") {
            client_options.release = Some(release.into());
        }
        if let Some(environment) = config.get_str("environment") {
            client_options.environment = Some(environment.into());
        }
        if let Some(sample_rate) = config.get_f32("sample-rate") {
            client_options.sample_rate = sample_rate;
        }
        if let Some(traces_sample_rate) = config.get_f32("traces-sample-rate") {
            client_options.traces_sample_rate = traces_sample_rate;
        }
        if let Some(max_breadcrumbs) = config.get_usize("max-breadcrumbs") {
            client_options.max_breadcrumbs = max_breadcrumbs;
        }
        if let Some(attach_stacktrace) = config.get_bool("attach-stacktrace") {
            client_options.attach_stacktrace = attach_stacktrace;
        }
        if let Some(send_default_pii) = config.get_bool("send-default-pii") {
            client_options.send_default_pii = send_default_pii;
        }
        if let Some(server_name) = config.get_str("server-name") {
            client_options.server_name = Some(server_name.into());
        }
        if let Some(in_app_include) = config.get_str_array("in-app-include") {
            client_options.in_app_include = in_app_include;
        }
        if let Some(in_app_exclude) = config.get_str_array("in-app-exclude") {
            client_options.in_app_exclude = in_app_exclude;
        }
        if let Some(default_integrations) = config.get_bool("default-integrations") {
            client_options.default_integrations = default_integrations;
        }
        if let Some(http_proxy) = config.get_str("http-proxy") {
            client_options.http_proxy = Some(http_proxy.into());
        }
        if let Some(https_proxy) = config.get_str("https-proxy") {
            client_options.https_proxy = Some(https_proxy.into());
        }
        if let Some(shutdown_timeout) = config.get_duration("shutdown-timeout") {
            client_options.shutdown_timeout = shutdown_timeout;
        }
        if let Some(accept_invalid_certs) = config.get_bool("accept-invalid-certs") {
            client_options.accept_invalid_certs = accept_invalid_certs;
        }
        if let Some(auto_session_tracking) = config.get_bool("auto-session-tracking") {
            client_options.auto_session_tracking = auto_session_tracking;
        }
        if let Some(session_mode) = config.get_str("session-mode") {
            client_options.session_mode = if session_mode.eq_ignore_ascii_case("application") {
                SessionMode::Application
            } else {
                SessionMode::Request
            };
        }
        if let Some(extra_border_frames) = config.get_str_array("extra-border-frames") {
            client_options.extra_border_frames = extra_border_frames;
        }
        if let Some(trim_backtraces) = config.get_bool("trim-backtraces") {
            client_options.trim_backtraces = trim_backtraces;
        }
        if let Some(user_agent) = config.get_str("user-agent") {
            client_options.user_agent = user_agent.into();
        }
    }

    let client_guard = sentry::init(client_options);
    SENTRY_CLIENT_GUARD
        .set(client_guard)
        .unwrap_or_else(|_| panic!("fail to set the guard for the sentry client"));
}

/// Sentry client guard.
static SENTRY_CLIENT_GUARD: OnceLock<ClientInitGuard> = OnceLock::new();
