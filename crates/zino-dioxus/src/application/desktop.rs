use dioxus::{
    desktop::{
        Config, WindowBuilder,
        WindowCloseBehaviour::*,
        muda::{self, AboutMetadata, Menu},
        tao::window::{Fullscreen, Icon, Theme},
    },
    prelude::*,
};
use dioxus_router::{components::Router, routable::Routable};
use image::{ImageReader, error::ImageError};
use std::{fmt::Display, fs, marker::PhantomData, path::Path, str::FromStr, time::Duration};
use tokio::runtime::Builder;
use url::Url;
use zino_core::{
    application::{AppType, Application, Plugin},
    error::Error,
    extension::TomlTableExt,
    schedule::AsyncScheduler,
};

/// A webview-based desktop renderer for the Dioxus VirtualDom.
pub struct Desktop<R> {
    /// Custom plugins.
    custom_plugins: Vec<Plugin>,
    /// Custom root element.
    custom_root: Option<fn() -> Element>,
    /// Custom menubar.
    custom_menu: Option<Menu>,
    /// A flag to enable the right-click context menu.
    enable_context_menu: bool,
    /// Phantom type of Dioxus router.
    phantom: PhantomData<R>,
}

impl<R> Desktop<R>
where
    R: Routable,
    <R as FromStr>::Err: Display,
{
    /// Renders the app root.
    #[inline]
    fn app_root() -> Element {
        Self::config_webview();
        rsx! { Router::<R> {} }
    }

    /// Sets a custom root element.
    #[inline]
    pub fn with_root(mut self, root: fn() -> Element) -> Self {
        self.custom_root = Some(root);
        self
    }

    /// Sets a custom menu bar.
    #[inline]
    pub fn with_menu(mut self, menu: Option<Menu>) -> Self {
        self.custom_menu = menu;
        self
    }

    /// Enables the right-click context menu.
    #[inline]
    pub fn enable_context_menu(mut self) -> Self {
        self.enable_context_menu = true;
        self
    }

    /// Configures the WebView.
    pub fn config_webview() {
        let app_state = Self::shared_state();
        if let Some(config) = app_state.get_config("webview") {
            let desktop_context = dioxus::desktop::window();
            if let Some(zoom) = config.get_f64("zoom") {
                if let Err(err) = desktop_context.webview.zoom(zoom) {
                    tracing::error!("fail to set the webview zoom level: {err}");
                }
            }
            if let Some(url) = config.get_str("url") {
                if let Err(err) = desktop_context.webview.load_url(url) {
                    tracing::error!("fail to load url: {err}");
                }
            }
        }
    }

    /// Formats a local path as the Dioxus href.
    pub fn format_local_path(path: &Path) -> String {
        let path = path.to_string_lossy();
        if cfg!(target_os = "windows") && !path.is_empty() {
            format!("http://dioxus.{}", path.replace('\\', "/"))
        } else {
            path.into_owned()
        }
    }

    /// Parses a resource URL.
    pub fn parse_resource_url(url: &str) -> Result<Url, Error> {
        if url.starts_with("http://") || url.starts_with("https://") || url.starts_with("file://") {
            url.parse().map_err(Error::from)
        } else if cfg!(target_os = "windows") && !url.is_empty() {
            format!("http://dioxus.{url}").parse().map_err(Error::from)
        } else {
            Err(Error::new(format!("invalid resource URL: {url}")))
        }
    }

    /// Returns the application metadata for the about dialog.
    pub fn about_metadata() -> AboutMetadata {
        let name = Self::name();
        let version = Self::version();
        let config = Self::config();
        let copyright = config.get_str("copyright").map(|s| s.to_owned());
        let license = config.get_str("license").map(|s| s.to_owned());
        let website = config.get_str("website").map(|s| s.to_owned());
        let icon = config.get_table("desktop").and_then(|config| {
            let icon = config.get_str("icon").unwrap_or("public/favicon.ico");
            let icon_file = Self::parse_path(icon);
            ImageReader::open(&icon_file)
                .ok()
                .and_then(|rdr| rdr.decode().ok())
                .and_then(|image| {
                    let width = image.width();
                    let height = image.height();
                    let bytes = image.into_bytes();
                    muda::Icon::from_rgba(bytes, width, height).ok()
                })
        });
        AboutMetadata {
            name: name.to_owned().into(),
            version: version.to_owned().into(),
            copyright,
            license,
            website,
            icon,
            ..AboutMetadata::default()
        }
    }
}

impl<R> Default for Desktop<R> {
    #[inline]
    fn default() -> Self {
        Self {
            custom_plugins: Vec::new(),
            custom_root: None,
            custom_menu: None,
            enable_context_menu: cfg!(debug_assertions),
            phantom: PhantomData,
        }
    }
}

impl<R> Application for Desktop<R>
where
    R: Routable,
    <R as FromStr>::Err: Display,
{
    type Routes = R;

    const APP_TYPE: AppType = AppType::Desktop;

    #[inline]
    fn register(self, _routes: Self::Routes) -> Self {
        self
    }

    #[inline]
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
            .expect("fail to build Tokio runtime for Dioxus desktop");
        let app_env = Self::env();
        runtime.block_on(async {
            Self::load().await;
            app_env.load_plugins(self.custom_plugins).await;
        });
        if scheduler.is_ready() {
            if scheduler.is_blocking() {
                runtime.spawn(async move {
                    if let Err(err) = scheduler.run().await {
                        tracing::error!("fail to run the async scheduler: {err}");
                    }
                });
            } else {
                runtime.spawn(async move {
                    loop {
                        scheduler.tick().await;

                        // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                        if let Some(duration) = scheduler.time_till_next_job() {
                            tokio::time::sleep(duration).await;
                        }
                    }
                });
            }
        }

        let app_name = Self::name();
        let app_version = Self::version();
        let app_state = Self::shared_state();

        // Window configuration
        let mut disable_menu = false;
        let mut window_title = app_name;
        let mut app_window = WindowBuilder::new()
            .with_title(app_name)
            .with_maximized(true)
            .with_focused(true);
        if let Some(config) = app_state.get_config("window") {
            if let Some(title) = config.get_str("title") {
                app_window = app_window.with_title(title);
                window_title = title;
            }
            if let Some(value) = config.get("disable-menu") {
                if value.as_bool().is_some_and(|b| b) {
                    disable_menu = true;
                } else if value.as_str().is_some_and(|s| s == "auto") {
                    disable_menu = !cfg!(target_os = "macos");
                }
            }
            if config.get_bool("fullscreen").is_some_and(|b| b) {
                app_window = app_window.with_fullscreen(Some(Fullscreen::Borderless(None)));
            }
            if let Some(maximized) = config.get_bool("maximized") {
                app_window = app_window.with_maximized(maximized);
            }
            if let Some(resizable) = config.get_bool("resizable") {
                app_window = app_window.with_resizable(resizable);
            }
            if let Some(minimizable) = config.get_bool("minimizable") {
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
            if let Some(focused) = config.get_bool("focused") {
                app_window = app_window.with_focused(focused);
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
            if let Some(protected) = config.get_bool("content-protection") {
                app_window = app_window.with_content_protection(protected);
            }
            if let Some(visible) = config.get_bool("visible-on-all-workspaces") {
                app_window = app_window.with_visible_on_all_workspaces(visible);
            }
            if let Some(theme) = config.get_str("theme") {
                let (theme, background_color) = if theme == "Dark" {
                    (Theme::Dark, (0, 0, 0, 255))
                } else {
                    (Theme::Light, (255, 255, 255, 255))
                };
                app_window = app_window
                    .with_theme(Some(theme))
                    .with_background_color(background_color);
            }
        }

        // Desktop configuration
        let mut desktop_config = Config::new()
            .with_window(app_window)
            .with_disable_context_menu(!self.enable_context_menu)
            .with_disable_drag_drop_handler(cfg!(target_os = "windows"))
            .with_menu(if disable_menu { None } else { self.custom_menu });
        if let Some(config) = app_state.get_config("desktop") {
            let mut custom_heads = Vec::new();
            custom_heads.push(r#"<meta charset="UTF-8">"#.to_owned());

            let icon = config.get_str("icon").unwrap_or("public/favicon.ico");
            let icon_file = Self::parse_path(icon);
            if icon_file.exists() {
                match ImageReader::open(&icon_file)
                    .map_err(ImageError::IoError)
                    .and_then(|reader| reader.decode())
                {
                    Ok(img) => {
                        let width = img.width();
                        let height = img.height();
                        let href = Self::format_local_path(&icon_file);
                        let head =
                            format!(r#"<link rel="icon" type="image/x-icon" href="{href}">"#);
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
                    let href = if style.starts_with("https://") || style.starts_with("http://") {
                        style.to_owned()
                    } else {
                        let style_file = Self::parse_path(style);
                        Self::format_local_path(&style_file)
                    };
                    let head = format!(r#"<link rel="stylesheet" href="{href}">"#);
                    custom_heads.push(head);
                }
            }
            if let Some(scripts) = config.get_str_array("scripts") {
                for script in scripts {
                    let src = if script.starts_with("https://") || script.starts_with("http://") {
                        script.to_owned()
                    } else {
                        let script_file = Self::parse_path(script);
                        Self::format_local_path(&script_file)
                    };
                    let head = format!(r#"<script src="{src}"></script>"#);
                    custom_heads.push(head);
                }
            }
            desktop_config = desktop_config.with_custom_head(custom_heads.join("\n"));

            if let Some(dir) = config.get_str("resource-dir") {
                desktop_config = desktop_config.with_resource_directory(Self::parse_path(dir));
            }
            if let Some(dir) = config.get_str("data-dir") {
                desktop_config = desktop_config.with_data_directory(Self::parse_path(dir));
            }
            if let Some(custom_index) = config.get_str("custom-index") {
                let index_file = Self::parse_path(custom_index);
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
            if let Some(behaviour) = config.get_str("close-behaviour") {
                let behaviour = match behaviour {
                    "CloseWindow" => CloseWindow,
                    "LastWindowHides" => LastWindowHides,
                    _ => LastWindowExitsApp,
                };
                desktop_config = desktop_config.with_close_behaviour(behaviour);
            }
        }

        tracing::warn!(
            app_env = app_env.as_str(),
            app_name,
            app_version,
            zino_version = env!("CARGO_PKG_VERSION"),
            "launch a window named `{window_title}`",
        );

        let vdom = VirtualDom::new(self.custom_root.unwrap_or(Self::app_root));
        runtime.block_on(tokio::task::unconstrained(async move {
            dioxus::desktop::launch::launch_virtual_dom_blocking(vdom, desktop_config)
        }));
    }
}
