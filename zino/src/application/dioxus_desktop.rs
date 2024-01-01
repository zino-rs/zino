use dioxus::prelude::*;
use dioxus_desktop::{
    tao::window::{Icon, Theme},
    Config, WindowBuilder,
};
use dioxus_router::{components::Router, routable::Routable};
use image::{error::ImageError, io::Reader};
use std::{fmt::Display, fs, marker::PhantomData, str::FromStr, time::Duration};
use tokio::runtime::Builder;
use zino_core::{application::Application, extension::TomlTableExt, schedule::AsyncScheduler, Map};

/// A webview-based desktop renderer for the Dioxus VirtualDom.
#[derive(Default)]
pub struct DioxusDesktop<R> {
    /// Phantom type of Dioxus router.
    phantom: PhantomData<R>,
}

impl<R> DioxusDesktop<R>
where
    R: Routable + Clone,
    <R as FromStr>::Err: Display,
{
    /// Renders the app root.
    fn app_root<'a>(cx: Scope<'a, &'static Map>) -> Element<'a> {
        render! { Router::<R> {} }
    }
}

impl<R> Application for DioxusDesktop<R>
where
    R: Routable + Clone,
    <R as FromStr>::Err: Display,
{
    type Routes = R;

    fn register(self, _routes: Self::Routes) -> Self {
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
        if scheduler.is_ready() {
            runtime.spawn(async move {
                loop {
                    scheduler.tick().await;

                    // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                    tokio::time::sleep(scheduler.time_till_next_job()).await;
                }
            });
        }
        runtime.block_on(async {
            Self::load().await;
        });

        let app_env = Self::env();
        let app_name = Self::name();
        let app_version = Self::version();
        let app_state = Self::shared_state();
        let project_dir = Self::project_dir();
        let in_prod_mode = app_env.is_prod();

        // Window configuration
        let mut window_title = app_name;
        let mut app_window = WindowBuilder::new()
            .with_title(app_name)
            .with_maximized(true)
            .with_decorations(true);
        if let Some(config) = app_state.get_config("window") {
            if let Some(title) = config.get_str("title") {
                app_window = app_window.with_title(title);
                window_title = title;
            }
            if let Some(maximizable) = config.get_bool("maximizable") {
                app_window = app_window.with_maximizable(maximizable);
            }
            if let Some(decorations) = config.get_bool("decorations") {
                app_window = app_window.with_decorations(decorations);
            }
            if let Some(theme) = config.get_str("theme") {
                let theme = match theme {
                    "Light" => Theme::Light,
                    "Dark" => Theme::Dark,
                    _ => Theme::default(),
                };
                app_window = app_window.with_theme(Some(theme));
            }
            if let Some(transparent) = config.get_bool("transparent") {
                app_window = app_window.with_transparent(transparent);
            }
        }

        // Desktop configuration
        let mut desktop_config = Config::new()
            .with_window(app_window)
            .with_disable_context_menu(in_prod_mode);
        if let Some(config) = app_state.get_config("desktop") {
            let mut custom_heads = Vec::new();
            if let Some(icon) = config.get_str("icon") {
                let icon_file = project_dir.join(icon);
                match Reader::open(&icon_file)
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
            if !custom_heads.is_empty() {
                desktop_config = desktop_config.with_custom_head(custom_heads.join("\n"));
            }
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
            if let Some(disable) = config.get_bool("disable-context-menu") {
                desktop_config = desktop_config.with_disable_context_menu(disable);
            }
            if let Some(name) = config.get_str("root-name") {
                desktop_config = desktop_config.with_root_name(name);
            }
        }

        tracing::warn!(
            app_env = app_env.as_str(),
            app_name,
            app_version,
            "launch a window named `{window_title}`",
        );

        dioxus_desktop::launch_with_props(Self::app_root, Self::state_data(), desktop_config);
    }
}
