//! Building HTML views using templates.

use crate::{application::Application, error::Error, extend::TomlTableExt, Map};
use std::{path::Path, sync::OnceLock};
use tera::{Context, Tera};

/// Renders a template with the given data using [`tera`](https://crates.io/crates/tera).
pub fn render(template_name: &str, data: Map) -> Result<String, Error> {
    let view_engine = SHARED_VIEW_ENGINE
        .get()
        .ok_or_else(|| Error::new("failed to get view engine"))?;
    let context = Context::from_value(data.into())?;
    view_engine
        .render(template_name, &context)
        .map_err(|err| err.into())
}

/// Intializes view engine.
pub(crate) fn init<APP: Application + ?Sized>() {
    let mut template_dir = "templates";
    if let Some(view) = APP::config().get_table("view") {
        if let Some(dir) = view.get_str("template-dir") {
            template_dir = dir;
        }
    }

    let template_dir = if Path::new(template_dir).exists() {
        template_dir.to_owned()
    } else {
        APP::project_dir()
            .join("templates")
            .to_string_lossy()
            .into()
    };
    let template_dir_glob = template_dir + "/**/*";
    let mut view_engine =
        Tera::new(template_dir_glob.as_str()).expect("failed to parse html templates");
    view_engine.autoescape_on(vec![".html", ".html.tera", ".tera"]);
    if APP::env() == "dev" {
        view_engine
            .full_reload()
            .expect("failed to reload html templates");
    }
    SHARED_VIEW_ENGINE
        .set(view_engine)
        .expect("failed to set view engine");
}

/// Shared view engine.
static SHARED_VIEW_ENGINE: OnceLock<Tera> = OnceLock::new();
