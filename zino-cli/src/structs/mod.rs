use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone)]
pub(crate) struct ZinoToml {
    #[serde(rename = "zli-config", default)]
    pub(crate) zli_config: ZliConfig,
    #[serde(default)]
    pub(crate) remote: Remote,
    #[serde(default)]
    pub(crate) acme: Acme,
}

impl Default for ZinoToml {
    fn default() -> Self {
        let toml_str = match std::fs::read_to_string("./zino.toml") {
            Ok(toml_str) => toml_str,
            Err(err) => {
                log::warn!("Failed to read config file: {}, using default config", err);
                "".to_string()
            }
        };
        toml::from_str(&toml_str).unwrap_or_else(|e| {
            log::error!("Failed to parse config file: {}, using default config", e);
            Self::default()
        })
    }
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Default, Clone)]
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

#[derive(Debug, Deserialize, Default, Clone)]
pub(crate) struct Acme {
    pub(crate) domain: Vec<String>,
    pub(crate) email: Vec<String>,
    #[serde(default = "default_cache_path")]
    pub(crate) cache: PathBuf,
    #[serde(default = "default_product_mode", rename = "product-mode")]
    pub(crate) product_mode: bool,
    #[serde(default = "default_listening_at", rename = "listening-at")]
    pub(crate) listening_at: u16,
    #[serde(default = "default_forward_to", rename = "forward-to")]
    pub(crate) forward_to: u16,
}
fn default_cache_path() -> PathBuf {
    PathBuf::from("default/cache/path")
}

fn default_product_mode() -> bool {
    false
}

fn default_listening_at() -> u16 {
    443
}
fn default_forward_to() -> u16 {
    6080
}
