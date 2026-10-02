use crate::error::Error;
use std::path::Path;
use toml::value::Table;

/// Reads the config from a local file.
pub(super) fn read_config_file(config_file: &Path, env: &str) -> Result<Table, Error> {
    let data = std::fs::read_to_string(config_file)?;
    let config_table = if config_file.extension().and_then(|s| s.to_str()) == Some("json") {
        serde_json::from_str(&data)?
    } else {
        data.parse()?
    };
    if let Some(file_name) = config_file.file_name().and_then(|s| s.to_str()) {
        tracing::info!(env, "load the config `{file_name}`");
    }
    Ok(config_table)
}
