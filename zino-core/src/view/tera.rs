use crate::{error::Error, state::State, Map};
use std::sync::OnceLock;
use tera::{Context, Tera};

/// Renders a template with the given data using [`tera`](https://crates.io/crates/tera).
pub fn render(template_name: &str, data: Map) -> Result<String, Error> {
    let view_engine = SHARED_VIEW_ENGINE
        .get()
        .ok_or_else(|| Error::new("fail to get the `tera` view engine"))?;
    let context = Context::from_value(data.into())?;
    view_engine
        .render(template_name, &context)
        .map_err(Error::from)
}

/// Loads templates.
pub(crate) fn load_templates(app_state: &'static State<Map>, template_dir: String) {
    let template_dir_glob = template_dir + "/**/*";
    let mut view_engine =
        Tera::new(template_dir_glob.as_str()).expect("fail to parse html templates");
    view_engine.autoescape_on(vec![".html", ".html.tera", ".tera"]);
    if app_state.env().is_dev() {
        view_engine
            .full_reload()
            .expect("fail to reload html templates");
    }
    SHARED_VIEW_ENGINE
        .set(view_engine)
        .expect("fail to set the `tera` view engine");
}

/// Shared view engine.
static SHARED_VIEW_ENGINE: OnceLock<Tera> = OnceLock::new();
