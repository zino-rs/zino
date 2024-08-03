use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::Dir;
use serde::Deserialize;
use std::{env, fs};
use toml_edit::{Array, DocumentMut as Document};
use zino::prelude::*;
use zino_core::error::Error;

/// Start the server.
#[derive(Parser)]
pub struct Serve {}

/// Resource directory.
static RESOURCE: Dir = include_dir::include_dir!("zino-cli/public");

/// Set configuration of the project
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
                .route("/generate_cargo_toml", post(generate_cargo_toml))])
            .run();
        Ok(())
    }
}

/// Returns the content of `Cargo.toml` file in current_dir
async fn get_current_cargo_toml() -> impl IntoResponse {
    fs::read_to_string("./Cargo.toml")
        .map(|content| content.into_response())
        .unwrap_or_else(|err| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("fail to read 'Cargo.toml' file: {err}"),
            )
                .into_response()
        })
}

/// Returns Html page.
async fn get_page(Path(file_name): Path<String>) -> impl IntoResponse {
    match RESOURCE.get_file(&file_name) {
        Some(file) => {
            let content = file.contents_utf8().unwrap_or_default();
            Html(content).into_response()
        }
        None => RESOURCE
            .get_file("404.html")
            .map(|not_found_page| {
                let not_found_page_content = not_found_page.contents_utf8().unwrap_or_default();
                (
                    axum::http::StatusCode::NOT_FOUND,
                    Html(not_found_page_content),
                )
                    .into_response()
            })
            .unwrap_or_else(|| {
                (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    "404.html not found, include_dir is corrupted. Try reinstall zli",
                )
                    .into_response()
            }),
    }
}

/// Returns current directory.
async fn get_current_dir() -> impl IntoResponse {
    env::current_dir()
        .map(|current_dir| {
            current_dir
                .to_str()
                .unwrap_or("fail to convert current_path to utf-8 string")
                .to_string()
                .into_response()
        })
        .unwrap_or_else(|err| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("fail to get current_dir: {err}"),
            )
                .into_response()
        })
}

/// Updates current directory.
async fn update_current_dir(Path(path): Path<String>) -> impl IntoResponse {
    env::set_current_dir(&path)
        .map(|_| {
            log::info!("Directory updated to: {}", path);
            axum::http::StatusCode::OK.into_response()
        })
        .unwrap_or_else(|err| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("fail to update current_dir: {err}"),
            )
                .into_response()
        })
}

/// Features struct.
#[derive(Debug, Deserialize)]
struct Features {
    zino_feature: Vec<String>,
    core_feature: Vec<String>,
}

/// Generates dependencies in `Cargo.toml` file.
async fn generate_cargo_toml(mut req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    let body = req.parse_body::<Features>().await?;

    let current_cargo_toml_content = fs::read_to_string("./Cargo.toml");
    if let Err(err) = current_cargo_toml_content {
        res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        res.set_data(&err.to_string());
        return Ok(res.into());
    }
    let current_cargo_toml = current_cargo_toml_content.unwrap().parse::<Document>();
    if let Err(err) = current_cargo_toml {
        res.set_code(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
        res.set_data(&err.to_string());
        return Ok(res.into());
    }
    let mut cargo_toml = current_cargo_toml.unwrap();

    if cargo_toml.get("dependencies").is_none() {
        cargo_toml["dependencies"] = toml_edit::table();
    }

    {
        // let mut zino_feature = Array::default();
        // for feature in body.zino_feature {
        //     zino_feature.push(feature);
        // }

        let zino_feature: Array = body.zino_feature.into_iter().collect();
        if cargo_toml["dependencies"].get("zino").is_none() {
            let mut zino_table = toml_edit::table();
            zino_table["version"] = toml_edit::value("0.24.3");
            zino_table["features"] = toml_edit::value(zino_feature);
            cargo_toml["dependencies"]["zino"] = zino_table;
        } else {
            cargo_toml["dependencies"]["zino"]["features"] = toml_edit::value(zino_feature);
        }

        // let mut core_feature = Array::default();
        // for feature in body.core_feature {
        //     core_feature.push(feature);
        // }

        let core_feature: Array = body.core_feature.into_iter().collect();
        if cargo_toml["dependencies"].get("zino-core").is_none() {
            let mut core_table = toml_edit::table();
            core_table["version"] = toml_edit::value("0.24.3");
            core_table["features"] = toml_edit::value(core_feature);
            cargo_toml["dependencies"]["zino-core"] = core_table;
        } else {
            cargo_toml["dependencies"]["zino-core"]["features"] = toml_edit::value(core_feature);
        }
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
