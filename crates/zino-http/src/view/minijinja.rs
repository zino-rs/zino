use convert_case::{Case, Casing};
use minijinja::{Environment, syntax::SyntaxConfig, value::Serde};
use std::sync::LazyLock;
use zino_core::{
    Map,
    application::{Agent, Application},
    error::Error,
    extension::TomlTableExt,
};

/// Renders a template with the given data using [`minijinja`](https://crates.io/crates/minijinja).
pub fn render(template_name: &str, data: Map) -> Result<String, Error> {
    let template = if template_name.contains('.') {
        SHARED_VIEW_ENGINE.get_template(template_name)?
    } else {
        let name = [template_name, ".jinja"].concat();
        SHARED_VIEW_ENGINE.get_template(&name)?
    };
    template.render(Serde(data)).map_err(Error::from)
}

/// Shared view engine.
static SHARED_VIEW_ENGINE: LazyLock<Environment> = LazyLock::new(|| {
    let app_state = Agent::shared_state();
    let mut template_dir = "templates";
    if let Some(view) = app_state.get_config("view") {
        if let Some(dir) = view.get_str("template-dir") {
            template_dir = dir;
        }
    }

    let app_env = app_state.env();
    let template_dir = Agent::parse_path(template_dir);
    let syntax = SyntaxConfig::builder()
        .trim_blocks(true)
        .lstrip_blocks(true)
        .build()
        .expect("fail to build the syntax");
    let mut view_engine = Environment::new();
    view_engine.set_syntax(syntax);
    view_engine.set_debug(app_env.is_dev());
    view_engine.set_loader(minijinja::path_loader(template_dir));
    view_engine.add_global("APP_ENV", app_env.as_str());
    for (key, value) in app_state.data() {
        if let Some(value) = value.as_str() {
            let key = key.replace('.', "_").to_case(Case::UpperSnake);
            view_engine.add_global(key, value);
        }
    }
    view_engine
});
