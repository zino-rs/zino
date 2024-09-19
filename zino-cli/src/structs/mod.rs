use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ZinoToml {
    #[serde(rename = "zli-config", default)]
    pub(crate) zli_config: ZliConfig,
    #[serde(default)]
    pub(crate) remote: Remote,
    #[serde(default)]
    pub(crate) acme: Acme,
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

#[derive(Debug, Deserialize,Default)]
pub(crate) struct Acme {
    pub(crate) domain: Vec<String>,
    pub(crate) email: Vec<String>,
    #[serde(default = "default_cache_path")]
    pub(crate) cache: PathBuf,
    #[serde(default = "default_product_mode", rename = "product-mode")]
    pub(crate) product_mode: bool,
    #[serde(default = "default_port")]
    pub(crate) port: u16,
}
fn default_cache_path() -> PathBuf {
    PathBuf::from("default/cache/path")
}

fn default_product_mode() -> bool {
    false
}

fn default_port() -> u16 {
    443
}
