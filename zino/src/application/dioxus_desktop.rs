use dioxus::prelude::*;
use dioxus_desktop::{tao::window::Theme, Config, WindowBuilder};
use dioxus_router::{components::Router, routable::Routable};
use std::{fmt::Display, marker::PhantomData, str::FromStr, time::Duration};
use tokio::runtime::Builder;
use zino_core::{
    application::Application,
    extension::TomlTableExt,
    schedule::{AsyncCronJob, Job, JobScheduler},
    Map,
};

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

    fn register_with(self, _server_name: &'static str, _routes: Self::Routes) -> Self {
        self
    }

    fn run(self, async_jobs: Vec<(&'static str, AsyncCronJob)>) {
        let runtime = Builder::new_multi_thread()
            .thread_keep_alive(Duration::from_secs(10))
            .thread_stack_size(2 * 1024 * 1024)
            .global_queue_interval(61)
            .enable_all()
            .build()
            .expect("fail to build Tokio runtime for `DioxusDesktop`");
        let mut scheduler = JobScheduler::new();
        for (cron_expr, exec) in async_jobs {
            scheduler.add(Job::new_async(cron_expr, exec));
        }
        runtime.spawn(async move {
            loop {
                scheduler.tick_async().await;

                // Cannot use `std::thread::sleep` because it blocks the Tokio runtime.
                tokio::time::sleep(scheduler.time_till_next_job()).await;
            }
        });

        let app_env = Self::env();
        let app_name = Self::name();
        let app_version = Self::version();
        let mut window_title = app_name;
        let mut app_window = WindowBuilder::new()
            .with_title(app_name)
            .with_maximized(true)
            .with_decorations(true);
        if let Some(config) = Self::shared_state().get_config("window") {
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
        tracing::warn!(
            app_env,
            app_name,
            app_version,
            "launch a window named `{window_title}`",
        );

        let app_config = Config::new().with_window(app_window);
        dioxus_desktop::launch_with_props(Self::app_root, Self::state_data(), app_config);
    }
}
