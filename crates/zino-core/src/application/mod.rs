//! High level abstractions for the application.
//!
//! # Examples
//!
//! ```rust,ignore
//! use casbin::prelude::*;
//! use std::sync::OnceLock;
//! use zino_core::application::{Application, Plugin};
//!
//! #[derive(Debug, Clone, Copy)]
//! pub struct Casbin;
//!
//! impl Casbin {
//!     pub fn init() -> Plugin {
//!         let loader = Box::pin(async {
//!             let model_file = "./config/casbin/model.conf";
//!             let policy_file = "./config/casbin/policy.csv";
//!             let enforcer = Enforcer::new(model_file, policy_file).await?;
//!             if CASBIN_ENFORCER.set(enforcer).is_err() {
//!                 tracing::error!("fail to initialize the Casbin enforcer");
//!             }
//!             Ok(())
//!         });
//!         Plugin::with_loader("casbin", loader)
//!     }
//! }
//!
//! static CASBIN_ENFORCER: OnceLock<Enforcer> = OnceLock::new();
//!
//! fn main() {
//!     zino::Cluster::boot()
//!         .add_plugin(Casbin::init())
//!         .run()
//! }
//! ```

use crate::{
    LazyLock, Map,
    datetime::DateTime,
    extension::{JsonObjectExt, TomlTableExt},
    schedule::{AsyncJobScheduler, AsyncScheduler, Scheduler},
    state::{Env, State},
};
use ahash::{HashMap, HashMapExt};
use std::{
    borrow::Cow,
    env, fs,
    path::{Component, Path, PathBuf},
    thread,
};
use toml::value::Table;

mod agent;
mod app_type;
mod plugin;
mod secret_key;
mod server_tag;
mod static_record;

#[cfg(feature = "http-client")]
pub(crate) mod http_client;

#[cfg(feature = "metrics")]
mod metrics_exporter;

#[cfg(feature = "preferences")]
mod preferences;

#[cfg(feature = "sentry")]
mod sentry_client;

#[cfg(feature = "tracing-subscriber")]
mod tracing_subscriber;

pub(crate) use secret_key::SECRET_KEY;

#[cfg(feature = "preferences")]
pub use preferences::Preferences;

#[cfg(feature = "http-client")]
use crate::{error::Error, extension::HeaderMapExt, trace::TraceContext};

pub use agent::Agent;
pub use app_type::AppType;
pub use plugin::Plugin;
pub use server_tag::ServerTag;
pub use static_record::StaticRecord;

/// Application interfaces.
pub trait Application {
    /// Routes.
    type Routes;

    /// Application type.
    const APP_TYPE: AppType;

    /// Registers default routes.
    fn register(self, routes: Self::Routes) -> Self;

    /// Runs the application with an optional scheduler for async jobs.
    fn run_with<T: AsyncScheduler + Send + 'static>(self, scheduler: T);

    /// Boots the application with the default initialization.
    fn boot() -> Self
    where
        Self: Default,
    {
        // Loads the `.env` file from the current directory or parents
        #[cfg(feature = "dotenv")]
        dotenvy::dotenv().ok();

        // Tracing subscriber
        #[cfg(feature = "tracing-subscriber")]
        tracing_subscriber::init::<Self>();

        // Secret keys
        secret_key::init::<Self>();

        // Metrics exporter
        #[cfg(feature = "metrics")]
        metrics_exporter::init::<Self>();

        // HTTP client
        #[cfg(feature = "http-client")]
        http_client::init::<Self>();

        // Initializes the directories to ensure that they are ready for use
        for path in SHARED_DIRS.values() {
            if !path.exists() {
                if let Err(err) = fs::create_dir_all(path) {
                    let path = path.display();
                    tracing::error!("fail to create the directory {path}: {err}");
                }
            }
        }

        Self::default()
    }

    /// Boots the application with a custom initialization.
    fn boot_with<F>(init: F) -> Self
    where
        Self: Default,
        F: FnOnce(&'static State<Map>),
    {
        let app = Self::boot();
        init(Self::shared_state());
        app
    }

    /// Registers routes with a server tag.
    #[inline]
    fn register_with(self, server_tag: ServerTag, routes: Self::Routes) -> Self
    where
        Self: Sized,
    {
        if server_tag == ServerTag::Debug {
            self.register(routes)
        } else {
            self
        }
    }

    /// Registers routes for debugger.
    #[inline]
    fn register_debug(self, routes: Self::Routes) -> Self
    where
        Self: Sized,
    {
        self.register_with(ServerTag::Debug, routes)
    }

    /// Adds a custom plugin.
    #[inline]
    fn add_plugin(self, plugin: Plugin) -> Self
    where
        Self: Sized,
    {
        tracing::info!(plugin_name = plugin.name());
        self
    }

    /// Returns a reference to the shared application state.
    #[inline]
    fn shared_state() -> &'static State<Map> {
        &SHARED_APP_STATE
    }

    /// Returns the application env.
    #[inline]
    fn env() -> &'static Env {
        SHARED_APP_STATE.env()
    }

    /// Returns a reference to the shared application config.
    #[inline]
    fn config() -> &'static Table {
        SHARED_APP_STATE.config()
    }

    /// Returns a reference to the shared application state data.
    #[inline]
    fn state_data() -> &'static Map {
        SHARED_APP_STATE.data()
    }

    /// Returns the application name.
    #[inline]
    fn name() -> &'static str {
        APP_NAME.as_ref()
    }

    /// Returns the application version.
    #[inline]
    fn version() -> &'static str {
        APP_VERSION.as_ref()
    }

    /// Returns the domain for the application.
    #[inline]
    fn domain() -> &'static str {
        APP_DOMAIN.as_ref()
    }

    /// Returns the secret key for the application.
    /// It should have at least 64 bytes.
    ///
    /// # Note
    ///
    /// This should only be used for internal services. Do not expose it to external users.
    #[inline]
    fn secret_key() -> &'static [u8] {
        SECRET_KEY.get().expect("fail to get the secret key")
    }

    /// Returns the project directory for the application.
    #[inline]
    fn project_dir() -> &'static PathBuf {
        &PROJECT_DIR
    }

    /// Returns the config directory for the application.
    ///
    /// # Note
    ///
    /// The default config directory is `${PROJECT_DIR}/config`.
    /// It can also be specified by the environment variable `ZINO_APP_CONFIG_DIR`.
    #[inline]
    fn config_dir() -> &'static PathBuf {
        &CONFIG_DIR
    }

    /// Returns the shared directory with a specific name,
    /// which is defined in the `dirs` table.
    ///
    /// # Examples
    ///
    /// ```toml
    /// [dirs]
    /// data = "/data/zino" # an absolute path
    /// cache = "~/zino/cache" # a path in the home dir
    /// assets = "local/assets" # a path in the project dir
    /// ```
    #[inline]
    fn shared_dir(name: &str) -> Cow<'_, PathBuf> {
        SHARED_DIRS
            .get(name)
            .map(Cow::Borrowed)
            .unwrap_or_else(|| Cow::Owned(Self::parse_path(name)))
    }

    /// Parses an absolute path, or a path relative to the home dir `~/` or project dir.
    #[inline]
    fn parse_path(path: &str) -> PathBuf {
        join_path(&PROJECT_DIR, path)
    }

    /// Spawns a new thread to run cron jobs.
    fn spawn<T>(self, mut scheduler: T) -> Self
    where
        Self: Sized,
        T: Scheduler + Send + 'static,
    {
        thread::spawn(move || {
            loop {
                scheduler.tick();
                if let Some(duration) = scheduler.time_till_next_job() {
                    thread::sleep(duration);
                }
            }
        });
        self
    }

    /// Runs the application with a default job scheduler.
    #[inline]
    fn run(self)
    where
        Self: Sized,
    {
        self.run_with(AsyncJobScheduler::default());
    }

    /// Loads resources after booting the application.
    #[inline]
    async fn load() {}

    /// Handles the graceful shutdown.
    #[inline]
    async fn shutdown() {}

    /// Makes an HTTP request to the provided URL.
    #[cfg(feature = "http-client")]
    async fn fetch(url: &str, options: Option<&Map>) -> Result<reqwest::Response, Error> {
        let mut trace_context = TraceContext::new();
        trace_context.record_trace_state();
        http_client::request_builder(url, options)?
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await
            .map_err(Error::from)
    }

    /// Makes an HTTP request to the provided URL and
    /// deserializes the response body via JSON.
    #[cfg(feature = "http-client")]
    async fn fetch_json<T: serde::de::DeserializeOwned>(
        url: &str,
        options: Option<&Map>,
    ) -> Result<T, Error> {
        let response = Self::fetch(url, options).await?.error_for_status()?;
        let data = if response.headers().has_json_content_type() {
            response.json().await?
        } else {
            let text = response.text().await?;
            serde_json::from_str(&text)?
        };
        Ok(data)
    }
}

/// Joins a path to the specific dir.
fn join_path(dir: &Path, path: &str) -> PathBuf {
    fn join_path_components(mut full_path: PathBuf, path: &str) -> PathBuf {
        for component in Path::new(path).components() {
            match component {
                Component::CurDir => (),
                Component::ParentDir => {
                    full_path.pop();
                }
                _ => {
                    full_path.push(component);
                }
            }
        }
        full_path
    }

    if path.starts_with('/') {
        path.into()
    } else if let Some(path) = path.strip_prefix("~/") {
        if let Some(home_dir) = dirs::home_dir() {
            join_path_components(home_dir, path)
        } else {
            join_path_components(dir.to_path_buf(), path)
        }
    } else {
        join_path_components(dir.to_path_buf(), path)
    }
}

/// App name.
static APP_NAME: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("name")
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_NAME")
                .expect("fail to get the environment variable `CARGO_PKG_NAME`")
                .leak()
        })
});

/// App version.
static APP_VERSION: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("version")
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_VERSION")
                .expect("fail to get the environment variable `CARGO_PKG_VERSION`")
                .leak()
        })
});

/// App domain.
static APP_DOMAIN: LazyLock<&'static str> = LazyLock::new(|| {
    SHARED_APP_STATE
        .config()
        .get_str("domain")
        .unwrap_or("localhost")
});

/// The project directory.
static PROJECT_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("CARGO_MANIFEST_DIR")
        .ok()
        .filter(|var| !var.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if cfg!(not(debug_assertions)) && cfg!(target_os = "macos") {
                if let Ok(mut path) = env::current_exe() {
                    path.pop();
                    if path.ends_with("Contents/MacOS") {
                        path.pop();
                        path.push("Resources");
                        if path.exists() && path.is_dir() {
                            return path;
                        }
                    }
                }
            }
            tracing::warn!(
                "fail to get the environment variable `CARGO_MANIFEST_DIR`; \
                    current directory will be used as the project directory"
            );
            env::current_dir()
                .expect("project directory does not exist or permissions are insufficient")
        })
});

/// The config directory.
static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    env::var("ZINO_APP_CONFIG_DIR")
        .ok()
        .filter(|var| !var.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PROJECT_DIR.join("config"))
});

/// Shared directories.
static SHARED_DIRS: LazyLock<HashMap<String, PathBuf>> = LazyLock::new(|| {
    let mut dirs = HashMap::new();
    if let Some(config) = SHARED_APP_STATE.get_config("dirs") {
        for (key, value) in config {
            if let Some(path) = value.as_str() {
                dirs.insert(key.to_owned(), join_path(&PROJECT_DIR, path));
            }
        }
    }
    dirs
});

/// Shared app state.
static SHARED_APP_STATE: LazyLock<State<Map>> = LazyLock::new(|| {
    let mut state = State::default();
    state.load_config();

    let config = state.config();
    let app_name = config
        .get_str("name")
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_NAME")
                .expect("fail to get the environment variable `CARGO_PKG_NAME`")
        });
    let app_version = config
        .get_str("version")
        .map(|s| s.to_owned())
        .unwrap_or_else(|| {
            env::var("CARGO_PKG_VERSION")
                .expect("fail to get the environment variable `CARGO_PKG_VERSION`")
        });

    let mut data = Map::new();
    data.upsert("app.name", app_name);
    data.upsert("app.version", app_version);
    data.upsert("app.booted_at", DateTime::now());
    state.set_data(data);
    state
});
