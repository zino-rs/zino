use std::sync::LazyLock;
use tera::{Context, Tera};
use zino_core::{
    Map,
    application::{Agent, Application},
    error::Error,
    extension::TomlTableExt,
};

/// Renders a template with the given data using [`tera`](https://crates.io/crates/tera).
pub fn render(template_name: &str, data: Map) -> Result<String, Error> {
    let context = Context::from_value(data.into())?;
    let content = if template_name.contains('.') {
        SHARED_VIEW_ENGINE.render(template_name, &context)
    } else {
        let name = [template_name, ".tera"].concat();
        SHARED_VIEW_ENGINE.render(template_name, &context)?
    };
    Ok(content)
}

/// Shared view engine.
static SHARED_VIEW_ENGINE: LazyLock<Tera> = LazyLock::new(|| {
    let app_state = Agent::shared_state();
    let mut template_dir = "templates";
    if let Some(view) = app_state.get_config("view") {
        if let Some(dir) = view.get_str("template-dir") {
            template_dir = dir;
        }
    }

    let template_dir = Agent::parse_path(template_dir);
    let dir_glob = template_dir.to_string_lossy().into_owned() + "/**/*";
    let mut view_engine = Tera::new(dir_glob.as_str()).expect("fail to parse html templates");
    view_engine.autoescape_on(vec![".html", ".html.tera", ".tera"]);
    if app_state.env().is_dev() {
        view_engine
            .full_reload()
            .expect("fail to reload html templates");
    }
    view_engine
});
