use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::Dir;
use std::{env, fs};
use toml_edit::Array;
use toml_edit::DocumentMut as Document;
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


/// Generates dependencies in `Cargo.toml` file.
async fn generate_cargo_toml(mut req: zino::Request) -> zino::Result {
    let mut res = zino::Response::default().context(&req);
    let body = req.parse_body::<Map>().await?;

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

    if let Some(dependencies) = cargo_toml.get_mut("dependencies") {
        let mut zino_table = toml_edit::table();
        zino_table["version"] = toml_edit::value("0.23.3");
        zino_table["features"] = toml_edit::value(Array::new());

        zino_table["features"] = toml_edit::value(Array::new());
        zino_table["features"].as_array_mut().unwrap().push(
            body["Framework"][0]
                .to_string()
                .to_lowercase()
                .trim_matches('"'),
        );
        for feature in body["zino-features"].as_array().unwrap() {
            zino_table["features"]
                .as_array_mut()
                .unwrap()
                .push(feature.to_string().trim_matches('"'));
        }

        dependencies["zino"] = zino_table;

        let mut zino_core_table = toml_edit::table();
        zino_core_table["version"] = toml_edit::value("0.24.3");
        zino_core_table["features"] = toml_edit::value(Array::new());
        zino_core_table["features"]
            .as_array_mut()
            .unwrap()
            .push(body["Database"][0].to_string().trim_matches('"'));
        for accessor_feature in body["Accessor"].as_array().unwrap() {
            zino_core_table["features"]
                .as_array_mut()
                .unwrap()
                .push(accessor_feature.to_string().trim_matches('"'));
        }
        for core_feature in body["Connector"].as_array().unwrap() {
            zino_core_table["features"]
                .as_array_mut()
                .unwrap()
                .push(core_feature.to_string().trim_matches('"'));
        }
        for core_feature in body["locale"].as_array().unwrap() {
            zino_core_table["features"]
                .as_array_mut()
                .unwrap()
                .push(core_feature.to_string().trim_matches('"'));
        }
        for core_feature in body["core-features"].as_array().unwrap() {
            zino_core_table["features"]
                .as_array_mut()
                .unwrap()
                .push(core_feature.to_string().trim_matches('"'));
        }

        dependencies["zino-core"] = zino_core_table;
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
