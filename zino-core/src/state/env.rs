use self::Env::*;
use crate::application::Plugin;
use std::fmt;

/// Application running environment.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Env {
    #[default]
    /// The `dev` environment.
    Dev,
    /// The `prod` environment.
    Prod,
    /// A custom environment.
    Custom(&'static str),
}

impl Env {
    /// Returns `true` if `self` is the `dev` environment.
    #[inline]
    pub fn is_dev(&self) -> bool {
        matches!(self, Dev)
    }

    /// Returns `true` if `self` is the `prod` environment.
    #[inline]
    pub fn is_prod(&self) -> bool {
        matches!(self, Prod)
    }

    /// Returns `self` as `&'static str`.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Dev => "dev",
            Prod => "prod",
            Custom(name) => name,
        }
    }

    /// Loads the plugins for the application.
    pub async fn load_plugins(&self, plugins: Vec<Plugin>) {
        let app_env = self.as_str();
        let plugin_names = plugins
            .iter()
            .map(|plugin| plugin.name())
            .collect::<Vec<_>>();
        for plugin in plugins {
            let plugin_name = plugin.name();
            if plugin.enabled(self) {
                if let Some(dependency) = plugin
                    .dependencies()
                    .iter()
                    .find(|dep| !plugin_names.contains(dep))
                {
                    tracing::error!(
                        app_env,
                        plugin_name,
                        "fail to find the dependency `{dependency}` for the plugin `{plugin_name}`",
                    );
                } else if let Err(err) = plugin.load().await {
                    tracing::error!(
                        app_env,
                        plugin_name,
                        "fail to load the plugin `{plugin_name}`: {err}",
                    );
                } else {
                    tracing::warn!(app_env, plugin_name, "loaded the plugin `{plugin_name}`",);
                }
            } else {
                tracing::error!(
                    app_env,
                    plugin_name,
                    "plugin `{plugin_name}` can not run in `{app_env}`",
                );
            }
        }
    }
}

impl fmt::Display for Env {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let env = self.as_str();
        write!(f, "{env}")
    }
}

impl From<&'static str> for Env {
    #[inline]
    fn from(env: &'static str) -> Self {
        match env {
            "dev" => Dev,
            "prod" => Prod,
            _ => Custom(env),
        }
    }
}
