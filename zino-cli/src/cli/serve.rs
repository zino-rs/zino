use zino::prelude::*;
use zino_core::error::Error;
use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::Dir;
use std::{env, fs};


/// Start the server.
#[derive(Parser)]
pub struct Serve {}

/// Resource directory.
static RESOURCE: Dir = include_dir::include_dir!("zino-cli/public");

impl Serve {
    /// Runs the `serve` subcommand.
    pub fn run(self) -> Result<(), Error> {
        println!("Starting the server...");

        env::set_var("CARGO_PKG_NAME", env!("CARGO_PKG_NAME"));
        env::set_var("CARGO_PKG_VERSION", env!("CARGO_PKG_VERSION"));

        zino::Cluster::boot()
            .register(vec![Router::new()
                .route("/:file_name", get(get_page))
                .route("/current_dir", get(get_current_dir))
                .route("/update_current_dir/:path", post(update_current_dir))
                .route("/get_current_cargo_toml", get(get_current_cargo_toml))])
            .run();

        Ok(())
    }
}

/// Returns the content of `Cargo.toml` file in current_dir
async fn get_current_cargo_toml() -> impl IntoResponse {
    match fs::read("./Cargo.toml") {
        Ok(content) => match String::from_utf8(content) {
            Ok(content) => content.into_response(),
            Err(err) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "fail to convert 'Cargo.toml' to string: {err}, make sure it's UTF-8 encoded"
                ),
            )
                .into_response(),
        },
        Err(err) => (
            axum::http::StatusCode::NOT_FOUND,
            format!(
                "Cargo.toml fill not found, make sure you are in a Rust project directory: {err}"
            ),
        )
            .into_response(),
    }
}

/// Returns Html page.
async fn get_page(Path(file_name): Path<String>) -> impl IntoResponse {
    match RESOURCE.get_file(file_name) {
        Some(file) => {
            let content = file.contents_utf8().unwrap_or_default();
            Html(content).into_response()
        }
        None => {
            match RESOURCE.get_file("404.html") {
                None => {
                    (
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "404.html not found, include_dir is corrupted. Try reinstall zli"
                    )
                        .into_response()
                }
                Some(not_found_page) => {
                    let not_found_page_content = not_found_page
                        .contents_utf8()
                        .unwrap_or_default();
                    (
                        axum::http::StatusCode::NOT_FOUND,
                        Html(not_found_page_content),
                    )
                        .into_response()
                }
            }
        }
    }
}

/// Returns current directory.
async fn get_current_dir() -> impl IntoResponse {
    match env::current_dir() {
        Ok(current_dir) => current_dir
            .to_str()
            .unwrap_or("fail to convert current_path to utf-8 string")
            .to_string()
            .into_response(),
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("fail to get current_dir: {err}"),
        )
            .into_response(),
    }
}

/// Updates current directory.
async fn update_current_dir(Path(path): Path<String>) -> impl IntoResponse {
    match env::set_current_dir(&path) {
        Ok(_) => {
            log::info!("Directory updated to: {}", path);
            "Directory updated".into_response()
        }
        Err(err) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            format!("fail to update current_dir: {err}"),
        )
            .into_response(),
    }
}
