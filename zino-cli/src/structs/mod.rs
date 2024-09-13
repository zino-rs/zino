use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ZinoToml {
    #[serde(rename = "zli-config", default)]
    pub(crate) zli_config: ZliConfig,
    #[serde(default)]
    pub(crate) remote: Remote,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ZliConfig {
    #[serde(
        rename = "refresh-interval",
        with = "humantime_serde",
        default = "default_refresh_interval"
    )]
    pub(crate) refresh_interval: std::time::Duration,
}

fn default_refresh_interval() -> std::time::Duration {
    std::time::Duration::from_secs(60)
}

impl Default for ZliConfig {
    fn default() -> Self {
        Self {
            refresh_interval: std::time::Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct Remote {
    #[serde(default = "default_remote_name")]
    pub(crate) name: String,
    #[serde(default = "default_remote_branch")]
    pub(crate) branch: String,
}

fn default_remote_name() -> String {
    "origin".to_string()
}

fn default_remote_branch() -> String {
    "main".to_string()
}
