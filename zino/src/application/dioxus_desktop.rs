use dioxus::prelude::*;
use dioxus_desktop::{
    tao::window::{Icon, Theme},
    Config, WindowBuilder,
};
use dioxus_router::{components::Router, routable::Routable};
use image::{error::ImageError, ImageReader};
use std::{fmt::Display, fs, marker::PhantomData, str::FromStr, time::Duration};
use tokio::runtime::Builder;
use zino_core::{
    application::{Application, Plugin},
    extension::TomlTableExt,
    schedule::AsyncScheduler,
};

/// A webview-based desktop renderer for the Dioxus VirtualDom.
#[derive(Default)]
pub struct DioxusDesktop<R> {
    /// Custom plugins.
    custom_plugins: Vec<Plugin>,
    /// Phantom type of Dioxus router.
    phantom: PhantomData<R>,
}

impl<R> DioxusDesktop<R>
where
    R: Routable,
    <R as FromStr>::Err: Display,
{
    /// Renders the app root.
    fn app_root() -> Element {
        rsx! { Router::<R> {} }
    }
}

impl<R> Application for DioxusDesktop<R>
where
    R: Routable,
    <R as FromStr>::Err: Display,
{
    type Routes = R;

    fn register(self, _routes: Self::Routes) -> Self {
        self
    }

    fn add_plugin(mut self, plugin: Plugin) -> Self {
        self.custom_plugins.push(plugin);
        self
    }

    fn run_with<T: AsyncScheduler + Send + 'static>(self, mut scheduler: T) {
        let runtime = Builder::new_multi_thread()
            .thread_keep_alive(Duration::from_secs(60))
            .thread_stack_size(2 * 1024 * 1024)
            .global_queue_interval(61)
            .enable_all()
            .build()
            .expect("fail to build Tokio runtime for `DioxusDesktop`");
        let app_env = Self::env();
        runtime.block_on(async {
            Self::load().await;
            super::load_plugins(self.custom_plugins, app_env).await;
        });
        if scheduler.is_ready() {
            runtime.spawn(async move {
                loop {
                    scheduler.tick().await;

                    // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                    tokio::time::sleep(scheduler.time_till_next_job()).await;
                }
            });
        }

        let app_name = Self::name();
        let app_version = Self::version();
        let app_state = Self::shared_state();
        let project_dir = Self::project_dir();
        let in_prod_mode = app_env.is_prod();

        // Window configuration
        let mut window_title = app_name;
        let mut app_window = WindowBuilder::new()
            .with_title(app_name)
            .with_maximized(true);
        if let Some(config) = app_state.get_config("window") {
            if let Some(title) = config.get_str("title") {
                app_window = app_window.with_title(title);
                window_title = title;
            }
            if let Some(resizable) = config.get_bool("resizable") {
                app_window = app_window.with_resizable(resizable);
            }
            if let Some(minimizable) = config.get_bool("maximizable") {
                app_window = app_window.with_minimizable(minimizable);
            }
            if let Some(maximizable) = config.get_bool("maximizable") {
                app_window = app_window.with_maximizable(maximizable);
            }
            if let Some(closable) = config.get_bool("closable") {
                app_window = app_window.with_closable(closable);
            }
            if let Some(visible) = config.get_bool("visible") {
                app_window = app_window.with_visible(visible);
            }
            if let Some(transparent) = config.get_bool("transparent") {
                app_window = app_window.with_transparent(transparent);
            }
            if let Some(decorations) = config.get_bool("decorations") {
                app_window = app_window.with_decorations(decorations);
            }
            if let Some(always_on_bottom) = config.get_bool("always-on-bottom") {
                app_window = app_window.with_always_on_bottom(always_on_bottom);
            }
            if let Some(always_on_top) = config.get_bool("always-on-top") {
                app_window = app_window.with_always_on_top(always_on_top);
            }
            if let Some(theme) = config.get_str("theme") {
                let theme = match theme {
                    "Light" => Theme::Light,
                    "Dark" => Theme::Dark,
                    _ => Theme::default(),
                };
                app_window = app_window.with_theme(Some(theme));
            }
        }

        // Desktop configuration
        let mut desktop_config = Config::new()
            .with_window(app_window)
            .with_disable_context_menu(in_prod_mode)
            .with_menu(None);
        if let Some(config) = app_state.get_config("desktop") {
            let mut custom_heads = Vec::new();
            custom_heads.push(r#"<meta charset="UTF-8">"#.to_owned());

            let icon = config.get_str("icon").unwrap_or("public/favicon.ico");
            let icon_file = project_dir.join(icon);
            if icon_file.exists() {
                match ImageReader::open(&icon_file)
                    .map_err(ImageError::IoError)
                    .and_then(|reader| reader.decode())
                {
                    Ok(img) => {
                        let width = img.width();
                        let height = img.height();
                        let head =
                            format!(r#"<link rel="icon" type="image/x-icon" href="{icon}">"#);
                        custom_heads.push(head);
                        match Icon::from_rgba(img.into_bytes(), width, height) {
                            Ok(icon) => {
                                desktop_config = desktop_config.with_icon(icon);
                            }
                            Err(err) => {
                                let icon_file = icon_file.display();
                                tracing::error!("fail to set the icon `{icon_file}`: {err}");
                            }
                        }
                    }
                    Err(err) => {
                        let icon_file = icon_file.display();
                        tracing::error!("fail to decode the icon file `{icon_file}`: {err}");
                    }
                }
            }
            if let Some(stylesheets) = config.get_str_array("stylesheets") {
                for style in stylesheets {
                    let head = format!(r#"<link rel="stylesheet" href="{style}">"#);
                    custom_heads.push(head);
                }
            }
            if let Some(scripts) = config.get_str_array("scripts") {
                for script in scripts {
                    let head = format!(r#"<script src="{script}"></script>"#);
                    custom_heads.push(head);
                }
            }
            desktop_config = desktop_config.with_custom_head(custom_heads.join("\n"));

            if let Some(dir) = config.get_str("resource-dir") {
                desktop_config = desktop_config.with_resource_directory(project_dir.join(dir));
            }
            if let Some(dir) = config.get_str("data-dir") {
                desktop_config = desktop_config.with_data_directory(project_dir.join(dir));
            }
            if let Some(custom_index) = config.get_str("custom-index") {
                let index_file = project_dir.join(custom_index);
                match fs::read_to_string(&index_file) {
                    Ok(custom_index) => {
                        desktop_config = desktop_config.with_custom_index(custom_index);
                    }
                    Err(err) => {
                        let index_file = index_file.display();
                        tracing::error!("fail to read the index html file `{index_file}`: {err}");
                    }
                }
            }
            if let Some(name) = config.get_str("root-name") {
                desktop_config = desktop_config.with_root_name(name);
            }
        }

        tracing::warn!(
            app_env = app_env.as_str(),
            app_name,
            app_version,
            zino_version = env!("CARGO_PKG_VERSION"),
            "launch a window named `{window_title}`",
        );

        dioxus_desktop::launch::launch(Self::app_root, Vec::new(), desktop_config);
    }
}
