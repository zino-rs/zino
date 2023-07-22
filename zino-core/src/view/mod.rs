//! Building HTML views using templates.

use crate::{application::Application, extension::TomlTableExt};
use std::path::Path;

cfg_if::cfg_if! {
    if #[cfg(feature = "view-tera")] {
        mod tera;

        use self::tera::load_templates;
        pub use self::tera::render;
    } else {
        mod minijinja;

        use self::minijinja::load_templates;
        pub use self::minijinja::render;
    }
}

/// Intializes view engine.
pub(crate) fn init<APP: Application + ?Sized>() {
    let app_state = APP::shared_state();
    let mut template_dir = "templates";
    if let Some(view) = app_state.get_config("view") && 
        let Some(dir) = view.get_str("template-dir")
    {
        template_dir = dir;
    }

    let template_dir = if Path::new(template_dir).exists() {
        template_dir.to_owned()
    } else {
        APP::project_dir()
            .join("templates")
            .to_string_lossy()
            .into()
    };
    load_templates(app_state, template_dir);
}
