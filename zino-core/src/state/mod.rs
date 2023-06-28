//! Application scoped state.

use crate::{application, crypto, encoding::base64, extension::TomlTableExt, format};
use std::{
    borrow::Cow,
    env, fs,
    net::{IpAddr, SocketAddr},
    sync::LazyLock,
};
use toml::value::Table;

/// A state is a record of the env, config and associated data.
#[derive(Debug, Clone)]
pub struct State<T = ()> {
    /// Environment.
    env: &'static str,
    /// Configuration.
    config: Table,
    /// Associated data.
    data: T,
}

impl<T> State<T> {
    /// Creates a new instance.
    #[inline]
    pub fn new(env: &'static str, data: T) -> Self {
        Self {
            env,
            config: Table::new(),
            data,
        }
    }

    /// Loads the config file according to the specific env.
    pub fn load_config(&mut self) {
        let env = self.env;
        let config_file = application::PROJECT_DIR.join(format!("./config/config.{env}.toml"));
        let config = match fs::read_to_string(&config_file) {
            Ok(value) => {
                tracing::warn!(env, "`config.{env}.toml` loaded");
                value.parse().unwrap_or_default()
            }
            Err(err) => {
                let config_file = config_file.to_string_lossy();
                tracing::error!("fail to read the config file `{config_file}`: {err}");
                Table::new()
            }
        };
        self.config = config;
    }

    /// Set the state data.
    #[inline]
    pub fn set_data(&mut self, data: T) {
        self.data = data;
    }

    /// Returns the env as `&str`.
    #[inline]
    pub fn env(&self) -> &'static str {
        self.env
    }

    /// Returns a reference to the config.
    #[inline]
    pub fn config(&self) -> &Table {
        &self.config
    }

    /// Returns a reference to the config corresponding to the `extension`.
    #[inline]
    pub fn get_extension_config(&self, extension: &str) -> Option<&Table> {
        self.config().get_table("extensions")?.get_table(extension)
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

    /// Returns a list of listeners as `Vec<SocketAddr>`.
    pub fn listeners(&self) -> Vec<SocketAddr> {
        let config = self.config();
        let mut listeners = Vec::new();

        // Main server
        let (main_host, main_port) = if let Some(main) = config.get_table("main") {
            let host = main
                .get_str("host")
                .and_then(|s| s.parse::<IpAddr>().ok())
                .unwrap_or(IpAddr::from([127, 0, 0, 1]));
            let port = main.get_u16("port").unwrap_or(6080);
            (host, port)
        } else {
            (IpAddr::from([127, 0, 0, 1]), 6080)
        };
        listeners.push((main_host, main_port).into());

        // Optional standbys
        if config.contains_key("standby") {
            let standbys = config
                .get_array("standby")
                .expect("the `standby` field should be an array of tables");
            for standby in standbys.iter().filter_map(|v| v.as_table()) {
                let standby_host = standby
                    .get_str("host")
                    .and_then(|s| s.parse::<IpAddr>().ok())
                    .expect("the `standby.host` field should be a str");
                let standby_port = standby
                    .get_u16("port")
                    .expect("the `standby.port` field should be an integer");
                listeners.push((standby_host, standby_port).into());
            }
        }

        listeners
    }
}

impl State {
    /// Returns a reference to the shared state.
    #[inline]
    pub fn shared() -> &'static Self {
        LazyLock::force(&SHARED_STATE)
    }

    /// Encrypts the password in the config.
    pub fn encrypt_password(config: &Table) -> Option<Cow<'_, str>> {
        let password = config.get_str("password")?;
        application::SECRET_KEY.get().and_then(|key| {
            if let Ok(data) = base64::decode(password) &&
                crypto::decrypt(key, &data).is_ok()
            {
                Some(password.into())
            } else {
                crypto::encrypt(key, password.as_bytes())
                    .ok()
                    .map(|bytes| base64::encode(bytes).into())
            }
        })
    }

    /// Decrypts the password in the config.
    pub fn decrypt_password(config: &Table) -> Option<Cow<'_, str>> {
        let password = config.get_str("password")?;
        if let Ok(data) = base64::decode(password) {
            if let Some(key) = application::SECRET_KEY.get() &&
                let Ok(plaintext) = crypto::decrypt(key, &data)
            {
                return Some(plaintext.into());
            }
        }
        if let Some(encrypted_password) = Self::encrypt_password(config).as_deref() {
            let num_chars = password.len() / 4;
            let masked_password = format::mask_text(password, num_chars, num_chars);
            tracing::warn!(
                encrypted_password,
                "raw passowrd `{masked_password}` should be encypted"
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
        State::new(&DEFAULT_ENV, T::default())
    }
}

/// Default env.
static DEFAULT_ENV: LazyLock<&'static str> = LazyLock::new(|| {
    let mut default_env = "dev";
    for arg in env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("--env=") {
            default_env = value.to_owned().leak();
        }
    }
    default_env
});

/// Shared application state.
static SHARED_STATE: LazyLock<State> = LazyLock::new(|| {
    let mut state = State::default();
    state.load_config();
    state
});
