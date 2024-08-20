use axum::{
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::Dir;
use serde::{Deserialize, Serialize};
use std::{env, fs};
use toml_edit::{Array, DocumentMut as Document};
use zino::prelude::*;
use zino_core::error::Error;

/// Start the server.
#[derive(Parser)]
pub struct Serve {}

/// Resource directory.
static RESOURCE: Dir = include_dir::include_dir!("zino-cli/public");

/// Set configuration of the project.
impl Serve {
    /// Runs the `serve` subcommand.
    pub fn run(self) -> Result<(), Error> {
        log::info!("Starting server at: 127.0.0.1:6080/zino-config.html");

        env::set_var("CARGO_PKG_NAME", env!("CARGO_PKG_NAME"));
        env::set_var("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"));

        zino::Cluster::boot()
            .register(vec![Router::new()
                .route("/:file_name", get(get_page))
                .route("/current_dir", get(get_current_dir))
                .route("/update_current_dir/:path", post(update_current_dir))
                .route("/get_current_cargo_toml", get(get_current_cargo_toml))
                .route("/generate_cargo_toml", post(generate_cargo_toml))
                .route("/save_cargo_toml", post(save_cargo_toml))
                .route("/get_current_features", get(get_current_features))])
            .run();
        Ok(())
    }
}

/// Returns the content of `Cargo.toml` file in the current directory.
async fn get_current_cargo_toml(req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    fs::read_to_string("./Cargo.toml")
        .map(|content| {
            res.set_content_type("application/json");
            res.set_data(&content);
        })
        .map_err(|err| {
            res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.set_data(&err.to_string());
        })
        .ok();
    Ok(res.into())
}

/// Returns the HTML page.
async fn get_page(req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default();
    let file_name: String = req.parse_param("file_name").unwrap_or_default();
    RESOURCE
        .get_file(&file_name)
        .map(|file| {
            let content = file.contents_utf8().unwrap_or_default();
            res.set_content_type(match file_name.split('.').last().unwrap_or("html") {
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                _ => "text/plain",
            });
            res.set_data(&content);
        })
        .or_else(|| {
            RESOURCE.get_file("404.html").map(|not_found_page| {
                let not_found_page_content = not_found_page
                    .contents_utf8()
                    .unwrap_or("404.html not found, include_dir is corrupted. Try reinstall zli");
                res.set_content_type("text/html");
                res.set_data(&not_found_page_content);
            })
        });
    Ok(res.into())
}

/// Returns the current directory.
async fn get_current_dir(req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    env::current_dir()
        .map(|current_dir| {
            res.set_code(axum::http::StatusCode::OK);
            res.set_data(
                &current_dir
                    .to_str()
                    .unwrap_or("fail to convert current_path to utf-8 string"),
            );
        })
        .unwrap_or_else(|err| {
            res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.set_data(&format!("fail to get current_dir: {}", err));
        });
    Ok(res.into())
}

/// Updates current directory.
async fn update_current_dir(req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    let path: String = req.parse_param("path").unwrap_or_default();
    env::set_current_dir(&path)
        .map(|_| {
            res.set_code(axum::http::StatusCode::OK);
            res.set_data(&format!("directory updated to: {}", path));
        })
        .unwrap_or_else(|err| {
            res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.set_data(&format!("fail to update current_dir: {}", err));
        });
    Ok(res.into())
}

/// Features struct.
#[derive(Debug, Serialize, Deserialize, Default)]
struct Features {
    zino_feature: Vec<String>,
    core_feature: Vec<String>,
}

impl Features {
    /// Get zino-features and zino-core-features from path to Cargo.toml
    fn from_path(path: &str) -> Self {
        let cargo_toml = fs::read_to_string(path)
            .unwrap_or_default()
            .parse::<Document>()
            .unwrap_or_default();

        if cargo_toml.get("dependencies").is_none() {
            return Self::default();
        }

        let zino_features = if cargo_toml["dependencies"].get("zino").is_none()
            || cargo_toml["dependencies"]["zino"].get("features").is_none()
        {
            vec![]
        } else {
            match cargo_toml["dependencies"]["zino"]["features"].as_array() {
                Some(features) => features
                    .iter()
                    .map(|f| f.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            }
        };

        let core_features = if cargo_toml["dependencies"].get("zino-core").is_none()
            || cargo_toml["dependencies"]["zino-core"]
                .get("features")
                .is_none()
        {
            vec![]
        } else {
            match cargo_toml["dependencies"]["zino-core"]["features"].as_array() {
                Some(features) => features
                    .iter()
                    .map(|f| f.as_str().unwrap_or_default().to_string())
                    .collect(),
                None => vec![],
            }
        };

        Self {
            zino_feature: zino_features,
            core_feature: core_features,
        }
    }
}

/// Generates zino-features and zino-core-features from user select options
async fn generate_cargo_toml(mut req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    let body = req.parse_body::<Features>().await?;

    let current_cargo_toml_content = match fs::read_to_string("./Cargo.toml") {
        Ok(content) => content,
        Err(err) => {
            res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.set_data(&err.to_string());
            return Ok(res.into());
        }
    };
    let mut cargo_toml = match current_cargo_toml_content.parse::<Document>() {
        Ok(doc) => doc,
        Err(err) => {
            res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.set_data(&err.to_string());
            return Ok(res.into());
        }
    };

    if cargo_toml.get("dependencies").is_none() {
        cargo_toml["dependencies"] = toml_edit::table();
    }

    let zino_feature: Array = body.zino_feature.into_iter().collect();
    if cargo_toml["dependencies"].get("zino").is_none() {
        let mut zino_table = toml_edit::table();
        zino_table["version"] = toml_edit::value("0.24.3");
        zino_table["features"] = toml_edit::value(zino_feature);
        cargo_toml["dependencies"]["zino"] = zino_table;
    } else {
        cargo_toml["dependencies"]["zino"]["features"] = toml_edit::value(zino_feature);
    }

    let core_feature: Array = body.core_feature.into_iter().collect();
    if cargo_toml["dependencies"].get("zino-core").is_none() {
        let mut core_table = toml_edit::table();
        core_table["version"] = toml_edit::value("0.24.3");
        core_table["features"] = toml_edit::value(core_feature);
        cargo_toml["dependencies"]["zino-core"] = core_table;
    } else {
        cargo_toml["dependencies"]["zino-core"]["features"] = toml_edit::value(core_feature);
    }

    let options = taplo::formatter::Options {
        compact_arrays: false,
        compact_inline_tables: false,
        column_width: 50,
        ..Default::default()
    };

    let formatted_toml = taplo::formatter::format(&cargo_toml.to_string(), options);

    res.set_content_type("application/json");
    res.set_data(&formatted_toml);
    Ok(res.into())
}

/// Returns a `Features` struct from current_dir/Cargo.toml.
async fn get_current_features(req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    let features = Features::from_path("./Cargo.toml");
    res.set_content_type("application/json");
    res.set_data(&features);
    Ok(res.into())
}

/// Saves the content of `Cargo.toml` file.
async fn save_cargo_toml(mut req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);

    let body = req.parse_body::<String>().await?;

    match fs::write("./Cargo.toml", body) {
        Ok(_) => {
            res.set_code(axum::http::StatusCode::OK);
        }
        Err(err) => {
            res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            res.set_data(&err.to_string());
        }
    }
    Ok(res.into())
}
