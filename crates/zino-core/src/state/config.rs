use crate::error::Error;
use std::path::Path;
use toml::value::Table;

/// Fetches the config from a URL.
#[cfg(feature = "http-client")]
pub(super) fn fetch_config_url(config_url: &str, env: &str) -> Result<Table, Error> {
    let res = reqwest::blocking::get(config_url)?;
    let config_table = if res
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| s.starts_with("application/json"))
    {
        res.json()?
    } else {
        res.text()?.parse()?
    };
    tracing::info!(env, "`{config_url}` fetched");
    Ok(config_table)
}

/// Reads the config from a local file.
pub(super) fn read_config_file(config_file: &Path, env: &str) -> Result<Table, Error> {
    let data = std::fs::read_to_string(config_file)?;
    let config_table = if config_file.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&data)?
    } else {
        data.parse()?
    };
    if let Some(file_name) = config_file.file_name().and_then(|s| s.to_str()) {
        tracing::info!(env, "`{file_name}` loaded");
    }
    Ok(config_table)
}
