use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use clap::Parser;
use include_dir::Dir;
use std::{env, fs};
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

/// Returns the content of `Cargo.toml` file in the current directory.
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

/// Returns the HTML page.
async fn get_page(Path(file_name): Path<String>) -> impl IntoResponse {
    match RESOURCE.get_file(file_name) {
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

/// Returns the current directory.
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
            log::info!("directory updated to: {}", path);
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
