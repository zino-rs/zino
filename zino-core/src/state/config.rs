use crate::error::Error;
use std::path::Path;
use toml::value::Table;

/// Fetches the config from a URL.
pub(super) fn fetch_config_url(config_url: &str, env: &str) -> Result<Table, Error> {
    let res = ureq::get(config_url).query("env", env).call()?;
    let config_table = match res.content_type() {
        "application/json" => {
            let data = res.into_string()?;
            serde_json::from_str(&data)?
        }
        "application/yaml" => {
            let data = res.into_string()?;
            serde_yaml::from_str(&data)?
        }
        _ => res.into_string()?.parse()?,
    };
    tracing::info!(env, "`{config_url}` fetched");
    Ok(config_table)
}

/// Reads the config from a local file.
pub(super) fn read_config_file(config_file: &Path, env: &str) -> Result<Table, Error> {
    let data = std::fs::read_to_string(config_file)?;
    let config_table = match config_file.extension().and_then(|s| s.to_str()) {
        Some("json") => serde_json::from_str(&data)?,
        Some("yaml" | "yml") => serde_yaml::from_str(&data)?,
        _ => data.parse()?,
    };
    if let Some(file_name) = config_file.file_name().and_then(|s| s.to_str()) {
        tracing::info!(env, "`{file_name}` loaded");
    }
    Ok(config_table)
}
