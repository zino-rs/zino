//! Application scoped state.
//!
//! # Examples
//!
//! ```toml
//! # config/config.dev.toml
//!
//! [redis]
//! host = "127.0.0.1"
//! port = 6379
//! database = "dbnum"
//! username = "some_user"
//! password = "hsfU4Y3aRbxVNuLpVG5T+wb9jIDdQyaUIiPgeQrP0ZRM1g"
//!
//! ```
//!
//! ```rust,ignore
//! //! src/extension/redis.rs
//!
//! use parking_lot::Mutex;
//! use redis::{Client, Connection, RedisResult};
//! use zino_core::{state::State, LazyLock};
//!
//! #[derive(Debug, Clone, Copy)]
//! pub struct Redis;
//!
//! impl Redis {
//!     #[inline]
//!     pub fn get_value(key: &str) -> RedisResult<String> {
//!         REDIS_CONNECTION.lock().get(key)
//!     }
//!
//!     #[inline]
//!     pub fn set_value(key: &str, value: &str, seconds: u64) -> RedisResult<()> {
//!         REDIS_CONNECTION.lock().set_ex(key, value, seconds)
//!     }
//! }
//!
//! static REDIS_CLIENT: LazyLock<Client> = LazyLock::new(|| {
//!     let config = State::shared()
//!         .get_config("redis")
//!         .expect("field `redis` should be a table");
//!     let database = config
//!         .get_str("database")
//!         .expect("field `database` should be a str");
//!     let authority = State::format_authority(config, Some(6379));
//!     let url = format!("redis://{authority}/{database}");
//!     Client::open(url)
//!         .expect("fail to create a connector to the redis server")
//! });
//!
//! static REDIS_CONNECTION: LazyLock<Mutex<Connection>> = LazyLock::new(|| {
//!     let connection = REDIS_CLIENT.get_connection()
//!         .expect("fail to establish a connection to the redis server");
//!     Mutex::new(connection)
//! });
//! ```

use crate::{
    LazyLock,
    application::{self, Agent, Application, ServerTag},
    crypto,
    encoding::base64,
    error::Error,
    extension::TomlTableExt,
    helper,
};
use convert_case::{Case, Casing};
use serde::de::DeserializeOwned;
use std::{
    borrow::Cow,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};
use toml::value::{Table, Value};

mod config;
mod data;
mod env;

pub use data::{Data, SharedData};
pub use env::Env;

/// A state is a record of the env, config and associated data.
#[derive(Debug, Clone)]
pub struct State<T = ()> {
    /// Environment.
    env: Env,
    /// Configuration.
    config: Table,
    /// Associated data.
    data: T,
}

impl<T> State<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(env: Env, data: T) -> Self {
        Self {
            env,
            config: Table::new(),
            data,
        }
    }

    /// Loads the config according to the specific env.
    ///
    /// # Note
    ///
    /// It supports the `json` or `toml` format of configuration source data,
    /// which can be specified by the environment variable `ZINO_APP_CONFIG_FORMAT`.
    /// By default, it reads the config from a local file. If `ZINO_APP_CONFIG_URL` is set,
    /// it will fetch the config from the URL instead.
    pub fn load_config(&mut self) {
        let env = self.env.as_str();
        let mut config_table = if let Ok(config_url) = std::env::var("ZINO_APP_CONFIG_URL") {
            #[cfg(feature = "http-client")]
            {
                config::fetch_config_url(&config_url, env).unwrap_or_else(|err| {
                    tracing::error!("fail to fetch the config url `{config_url}`: {err}");
                    Table::new()
                })
            }
            #[cfg(not(feature = "http-client"))]
            {
                tracing::error!("cannot fetch the config url `{config_url}`");
                Table::new()
            }
        } else {
            let format = std::env::var("ZINO_APP_CONFIG_FORMAT")
                .map(|s| s.to_ascii_lowercase())
                .unwrap_or_else(|_| "toml".to_owned());
            let config_dir = Agent::config_dir();
            if config_dir.exists() {
                let config_file = format!("config.{env}.{format}");
                let config_file_path = config_dir.join(&config_file);
                config::read_config_file(&config_file_path, env).unwrap_or_else(|err| {
                    tracing::error!("fail to read the config file `{config_file}`: {err}");
                    Table::new()
                })
            } else {
                tracing::warn!("no config file found in `{}`", config_dir.display());
                Table::new()
            }
        };

        // Merges the environment variables for the config table.
        for (key, value) in config_table.iter_mut() {
            let Value::Table(table) = value else {
                continue;
            };
            let prefix = key.to_ascii_uppercase();
            for (key, value) in table.iter_mut() {
                let name = format!("ZINO_{}_{}", &prefix, key.to_case(Case::Constant));
                let Ok(s) = std::env::var(&name) else {
                    continue;
                };
                let result = match value.type_str() {
                    "string" => Ok(Value::String(s)),
                    "integer" => s.parse().map(Value::Integer).map_err(Error::from),
                    "float" => s.parse().map(Value::Float).map_err(Error::from),
                    "boolean" => s.parse().map(Value::Boolean).map_err(Error::from),
                    "datetime" => s.parse().map(Value::Datetime).map_err(Error::from),
                    _ => Err(Error::new(format!("cannot parse non-primitive value: {s}"))),
                };
                match result {
                    Ok(v) => *value = v,
                    Err(err) => tracing::error!("invalid environment variable `{name}`: {err}"),
                }
            }
        }

        self.config = config_table;
    }

    /// Set the state data.
    #[inline]
    pub fn set_data(&mut self, data: T) {
        self.data = data;
    }

    /// Returns the env.
    #[inline]
    pub fn env(&self) -> &Env {
        &self.env
    }

    /// Returns a reference to the config.
    #[inline]
    pub fn config(&self) -> &Table {
        &self.config
    }

    /// Returns a reference to the config corresponding to the `key`.
    #[inline]
    pub fn get_config(&self, key: &str) -> Option<&Table> {
        self.config().get_table(key)
    }

    /// Returns a reference to the config corresponding to the `extension`.
    #[inline]
    pub fn get_extension_config(&self, extension: &str) -> Option<&Table> {
        self.config().get_table("extensions")?.get_table(extension)
    }

    /// Parses the config as an instance of `C` corresponding to the `key`.
    #[inline]
    pub fn parse_config<C: DeserializeOwned>(&self, key: &str) -> Option<Result<C, Error>> {
        self.get_config(key)
            .map(|t| serde_json::from_value(t.to_map().into()).map_err(Error::from))
    }

    /// Parses the config as an instance of `C` corresponding to the `extension`.
    #[inline]
    pub fn parse_extension_config<C: DeserializeOwned>(
        &self,
        extension: &str,
    ) -> Option<Result<C, Error>> {
        self.get_extension_config(extension)
            .map(|t| serde_json::from_value(t.to_map().into()).map_err(Error::from))
    }

    /// Returns a reference to the data.
    #[inline]
    pub fn data(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Returns a list of listeners.
    pub fn listeners(&self) -> Vec<(ServerTag, SocketAddr)> {
        let config = self.config();
        let mut listeners = Vec::new();

        // Debug server
        if let Some(debug_server) = config.get_table("debug") {
            let debug_host = debug_server
                .get_str("host")
                .and_then(|s| s.parse::<IpAddr>().ok())
                .expect("field `debug.host` should be a str");
            let debug_port = debug_server
                .get_u16("port")
                .expect("field `debug.port` should be an integer");
            listeners.push((ServerTag::Debug, (debug_host, debug_port).into()));
        }

        // Main server
        if let Some(main_server) = config.get_table("main") {
            let main_host = main_server
                .get_str("host")
                .and_then(|s| s.parse::<IpAddr>().ok())
                .expect("field `main.host` should be a str");
            let main_port = main_server
                .get_u16("port")
                .expect("field `main.port` should be an integer");
            listeners.push((ServerTag::Main, (main_host, main_port).into()));
        }

        // Standbys
        if config.contains_key("standby") {
            let standbys = config
                .get_array("standby")
                .expect("field `standby` should be an array of tables");
            for standby in standbys.iter().filter_map(|v| v.as_table()) {
                let server_tag = standby.get_str("tag").unwrap_or("standby");
                let standby_host = standby
                    .get_str("host")
                    .and_then(|s| s.parse::<IpAddr>().ok())
                    .expect("field `standby.host` should be a str");
                let standby_port = standby
                    .get_u16("port")
                    .expect("field `standby.port` should be an integer");
                listeners.push((server_tag.into(), (standby_host, standby_port).into()));
            }
        }

        // Ensure that there is at least one listener
        if listeners.is_empty() {
            listeners.push((ServerTag::Main, (Ipv4Addr::LOCALHOST, 6080).into()));
        }

        listeners
    }
}

impl State {
    /// Returns a reference to the shared state.
    #[inline]
    pub fn shared() -> &'static Self {
        &SHARED_STATE
    }

    /// Encrypts the password in the config.
    pub fn encrypt_password(config: &Table) -> Option<Cow<'_, str>> {
        let password = config.get_str("password")?;
        application::SECRET_KEY.get().and_then(|key| {
            if base64::decode(password).is_ok_and(|data| crypto::decrypt(&data, key).is_ok()) {
                Some(password.into())
            } else {
                crypto::encrypt(password.as_bytes(), key)
                    .ok()
                    .map(|bytes| base64::encode(bytes).into())
            }
        })
    }

    /// Decrypts the password in the config.
    pub fn decrypt_password(config: &Table) -> Option<Cow<'_, str>> {
        let password = config.get_str("password")?;
        if let Ok(data) = base64::decode(password) {
            if let Some(key) = application::SECRET_KEY.get() {
                if let Ok(plaintext) = crypto::decrypt(&data, key) {
                    return Some(String::from_utf8_lossy(&plaintext).into_owned().into());
                }
            }
        }
        if let Some(encrypted_password) = Self::encrypt_password(config).as_deref() {
            let num_chars = password.len() / 4;
            let masked_password = helper::mask_text(password, num_chars, num_chars);
            tracing::warn!(
                encrypted_password,
                "raw password `{masked_password}` should be encypted"
            );
        }
        Some(password.into())
    }

    /// Formats the authority in the config.
    /// An authority can contain a username, password, host, and port number,
    /// which is formated as `{username}:{password}@{host}:{port}`.
    pub fn format_authority(config: &Table, default_port: Option<u16>) -> String {
        let mut authority = String::new();

        // Username
        let username = config.get_str("username").unwrap_or_default();
        authority += username;

        // Password
        if let Some(password) = Self::decrypt_password(config) {
            authority += &format!(":{password}@");
        }

        // Host
        let host = config.get_str("host").unwrap_or("localhost");
        authority += host;

        // Port
        if let Some(port) = config.get_u16("port").or(default_port) {
            authority += &format!(":{port}");
        }

        authority
    }
}

impl<T: Default> Default for State<T> {
    #[inline]
    fn default() -> Self {
        State::new(*DEFAULT_ENV, T::default())
    }
}

/// Default env.
static DEFAULT_ENV: LazyLock<Env> = LazyLock::new(|| {
    for arg in std::env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("--env=") {
            let env: &'static str = value.to_owned().leak();
            return env.into();
        }
    }
    if let Ok(value) = std::env::var("ZINO_APP_ENV") {
        let env: &'static str = value.to_owned().leak();
        return env.into();
    }
    if cfg!(debug_assertions) {
        Env::Dev
    } else {
        Env::Prod
    }
});

/// Shared application state.
static SHARED_STATE: LazyLock<State> = LazyLock::new(|| {
    let mut state = State::default();
    state.load_config();
    state
});
