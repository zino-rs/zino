use zino_core::{application::Plugin, state::Env};

/// Loads the plugins for the application.
pub(super) async fn load_plugins(plugins: Vec<Plugin>, app_env: &Env) {
    let plugin_names = plugins
        .iter()
        .map(|plugin| plugin.name())
        .collect::<Vec<_>>();
    for plugin in plugins {
        let plugin_name = plugin.name();
        if plugin.enabled(app_env) {
            if let Some(dependency) = plugin
                .dependencies()
                .iter()
                .find(|dep| !plugin_names.contains(dep))
            {
                tracing::error!(
                    app_env = app_env.as_str(),
                    plugin_name,
                    "fail to find the dependency `{dependency}` for the plugin `{plugin_name}`",
                );
            } else if let Err(err) = plugin.load().await {
                tracing::error!(
                    app_env = app_env.as_str(),
                    plugin_name,
                    "fail to load the plugin `{plugin_name}`: {err}",
                );
            } else {
                tracing::warn!(
                    app_env = app_env.as_str(),
                    plugin_name,
                    "loaded the plugin `{plugin_name}`",
                );
            }
        } else {
            tracing::error!(
                app_env = app_env.as_str(),
                plugin_name,
                "plugin `{plugin_name}` can not run in `{app_env}`",
            );
        }
    }
}
