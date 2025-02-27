//! Converting TOML files to amis schemas.

use std::{fs, path::Path};
use toml::{Table, Value};
use zino_core::{
    Map,
    error::Error,
    extension::{JsonObjectExt, TomlValueExt},
    json,
};

/// A list of page schema nodes.
const PAGE_SCHEMA_NODES: [&str; 5] = ["body", "aside", "toolbar", "title", "subTitle"];

/// Compiles the amis config files.
pub(crate) fn compile(config_dir: &Path, output_dir: &Path) -> Result<(), Error> {
    generate_schemas(config_dir, output_dir, None)
}

/// Outputs the amis schemas in the directory.
fn generate_schemas(
    config_dir: &Path,
    output_dir: &Path,
    route_name: Option<&str>,
) -> Result<(), Error> {
    let mut definitions = Map::new();
    let mut page_component = Map::new();
    page_component.upsert("type", "page");
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path.join("page.toml").exists() {
                if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                    let output_name = match route_name {
                        Some(name) => [name, dir_name].join("_"),
                        None => dir_name.to_owned(),
                    };
                    generate_schemas(&path, output_dir, Some(&output_name))?;
                }
            }
        } else {
            let config = read_config_file(&path);
            if entry.file_name() == "page.toml" {
                for (key, value) in config {
                    if PAGE_SCHEMA_NODES.contains(&key.as_str()) {
                        match value {
                            Value::String(s) => {
                                page_component.upsert(key, s);
                            }
                            Value::Table(table) => {
                                let schema = parse_schema(config_dir, table);
                                page_component.upsert(key, schema);
                            }
                            Value::Array(vec) => {
                                let schema_array = vec
                                    .into_iter()
                                    .filter_map(|value| {
                                        if let Value::Table(table) = value {
                                            Some(parse_schema(config_dir, table))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>();
                                page_component.upsert(key, schema_array);
                            }
                            _ => (),
                        }
                    } else if key == "definitions" {
                        if let Value::Table(config) = value {
                            for (key, value) in config {
                                if let Value::Table(table) = value {
                                    let schema = parse_schema(config_dir, table);
                                    definitions.upsert(key, schema);
                                }
                            }
                        }
                    } else {
                        page_component.upsert(key, value.to_json_value());
                    }
                }
            }
        }
    }
    if !definitions.is_empty() {
        page_component.upsert("definitions", definitions);
    }

    let bytes = serde_json::to_vec_pretty(&json!({
        "schema": page_component,
        "props": {},
    }))?;
    let output_file = [route_name.unwrap_or("index"), ".json"].concat();
    fs::write(output_dir.join(output_file), bytes)?;
    Ok(())
}

/// Parses a schema node from a TOML table.
fn parse_schema(config_dir: &Path, mut config: Table) -> Map {
    let mut schema = if let Some(Value::String(path)) = config.remove("include") {
        let config_file = config_dir.join(path).with_extension("toml");
        let config = read_config_file(&config_file);
        let config_dir = config_file.parent().unwrap_or(config_dir);
        parse_schema(config_dir, config)
    } else {
        Map::new()
    };
    for (key, value) in config {
        schema.upsert(key, value.to_json_value());
    }
    schema
}

/// Reads a config file as a TOML table.
fn read_config_file(path: &Path) -> Table {
    fs::read_to_string(path)
        .unwrap_or_else(|err| {
            let amis_file = path.display();
            panic!("fail to read the amis file `{amis_file}`: {err}");
        })
        .parse::<Table>()
        .expect("fail to parse the amis file as a TOML table")
}
