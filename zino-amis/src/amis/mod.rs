//! Converting TOML files to amis schemas.

use std::{fs, path::Path};
use toml::{Table, Value};
use zino_core::{
    error::Error,
    extension::{JsonObjectExt, TomlValueExt},
    json, Map,
};

/// A list of page schema nodes.
const PAGE_SCHEMA_NODES: [&str; 5] = ["body", "aside", "toolbar", "title", "subTitle"];

/// Outputs amis schemas in the directory.
pub(crate) fn generate_schema(
    config_dir: &Path,
    output_dir: &Path,
    route_name: Option<&str>,
) -> Result<(), Error> {
    let mut page_component = Map::new();
    page_component.upsert("type", "page");
    for entry in fs::read_dir(config_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if let Some(dir_name) = path.file_name().and_then(|s| s.to_str()) {
                let output_name = match route_name {
                    Some(name) => [name, dir_name].join("_"),
                    None => dir_name.to_owned(),
                };
                generate_schema(&path, output_dir, Some(&output_name))?;
            }
        } else {
            let config = fs::read_to_string(&path)
                .unwrap_or_else(|err| {
                    let amis_file = path.display();
                    panic!("fail to read the amis file `{amis_file}`: {err}");
                })
                .parse::<Table>()
                .expect("fail to parse the amis file as a TOML table");
            if entry.file_name() == "page.toml" {
                for (key, value) in config {
                    if PAGE_SCHEMA_NODES.contains(&key.as_str()) {
                        match value {
                            Value::String(s) => {
                                page_component.upsert(key, s);
                            }
                            Value::Table(table) => {
                                let schema = parse_schema(table);
                                page_component.upsert(key, schema);
                            }
                            Value::Array(vec) => {
                                let schema_array = vec
                                    .into_iter()
                                    .filter_map(|value| {
                                        if let Value::Table(table) = value {
                                            Some(parse_schema(table))
                                        } else {
                                            None
                                        }
                                    })
                                    .collect::<Vec<_>>();
                                page_component.upsert(key, schema_array);
                            }
                            _ => (),
                        }
                    } else {
                        page_component.upsert(key, value.to_json_value());
                    }
                }
            }
        }
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
fn parse_schema(config: Table) -> Map {
    let mut schema = Map::new();
    for (key, value) in config {
        schema.upsert(key, value.to_json_value());
    }
    schema
}
