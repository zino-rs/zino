use crate::{error::Error, state::State, Map};
use convert_case::{Case, Casing};
use minijinja::Environment;
use std::sync::OnceLock;

/// Renders a template with the given data using [`minijinja`](https://crates.io/crates/minijinja).
pub fn render(template_name: &str, data: Map) -> Result<String, Error> {
    let view_engine = SHARED_VIEW_ENGINE
        .get()
        .ok_or_else(|| Error::new("fail to get the `jinja` view engine"))?;
    let template = view_engine.get_template(template_name)?;
    template.render(data).map_err(Error::from)
}

/// Loads templates.
pub(crate) fn load_templates(app_state: &'static State<Map>, template_dir: String) {
    let mut view_engine = Environment::new();
    let app_env = app_state.env();
    view_engine.set_debug(app_env == "dev");
    view_engine.set_loader(minijinja::path_loader(template_dir));
    view_engine.add_global("APP_ENV", app_env);
    for (key, value) in app_state.data() {
        if let Some(value) = value.as_str() {
            let key = key.replace('.', "_").to_case(Case::UpperSnake);
            view_engine.add_global(key, value);
        }
    }
    SHARED_VIEW_ENGINE
        .set(view_engine)
        .expect("fail to set the `jinja` view engine");
}

/// Shared view engine.
static SHARED_VIEW_ENGINE: OnceLock<Environment> = OnceLock::new();
