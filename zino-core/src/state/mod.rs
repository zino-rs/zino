//! Application or request scoped state.

use crate::{application, crypto, extend::TomlTableExt, Map};
use base64_simd::STANDARD_NO_PAD;
use std::{
    borrow::Cow,
    env, fs,
    net::{IpAddr, SocketAddr},
    sync::LazyLock,
};
use toml::value::{Table, Value};

/// A state is a record of the env, config and associated data.
#[derive(Debug, Clone)]
pub struct State {
    /// Environment.
    env: &'static str,
    /// Configuration.
    config: Table,
    /// Associated data.
    data: Map,
}

impl State {
    /// Creates a new instance.
    #[inline]
    pub fn new(env: &'static str) -> Self {
        Self {
            env,
            config: Table::new(),
            data: Map::new(),
        }
    }

    /// Loads the config file according to the specific env.
    pub fn load_config(&mut self) {
        let env = self.env;
        let config_file = application::PROJECT_DIR.join(format!("./config/config.{env}.toml"));
        let config: Value = fs::read_to_string(&config_file)
            .unwrap_or_else(|err| {
                let config_file = config_file.to_string_lossy();
                panic!("failed to read the config file `{config_file}`: {err}");
            })
            .parse()
            .expect("failed to parse toml value");
        match config {
            Value::Table(table) => self.config = table,
            _ => panic!("toml config file should be a table"),
        }
    }

    /// Set the state data.
    #[inline]
    pub fn set_data(&mut self, data: Map) {
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

    /// Returns a reference to the data.
    #[inline]
    pub fn data(&self) -> &Map {
        &self.data
    }

    /// Returns a mutable reference to the data.
    #[inline]
    pub fn data_mut(&mut self) -> &mut Map {
        &mut self.data
    }

    /// Returns a list of listeners as `Vec<SocketAddr>`.
    pub fn listeners(&self) -> Vec<SocketAddr> {
        let config = self.config();
        let mut listeners = Vec::new();

        // Main server.
        let main = config
            .get_table("main")
            .expect("the `main` field should be a table");
        let main_host = main
            .get_str("host")
            .and_then(|s| s.parse::<IpAddr>().ok())
            .expect("the `main.host` field should be an IP address");
        let main_port = main
            .get_u16("port")
            .expect("the `main.port` field should be an integer");
        listeners.push((main_host, main_port).into());

        // Standbys.
        let standbys = config
            .get_array("standby")
            .expect("the `standby` field should be an array of tables");
        for standby in standbys {
            if standby.is_table() {
                let standby = standby
                    .as_table()
                    .expect("the `standby` field should be a table");
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

    /// Encrypts the password in the config.
    pub fn encrypt_password(config: &Table) -> Option<Cow<'_, str>> {
        let password = config.get_str("password")?;
        application::SECRET_KEY.get().and_then(|key| {
            if let Ok(data) = STANDARD_NO_PAD.decode_to_vec(password) &&
                crypto::decrypt(key, &data).is_ok()
            {
                Some(password.into())
            } else {
                crypto::encrypt(key, password.as_bytes())
                    .inspect_err(|_| tracing::error!("failed to encrypt the password"))
                    .ok()
                    .map(|bytes| STANDARD_NO_PAD.encode_to_string(bytes).into())
            }
        })
    }

    /// Decrypts the password in the config.
    pub fn decrypt_password(config: &Table) -> Option<Cow<'_, str>> {
        let password = config.get_str("password")?;
        if let Ok(data) = STANDARD_NO_PAD.decode_to_vec(password) {
            if let Some(key) = application::SECRET_KEY.get() &&
                let Ok(plaintext) = crypto::decrypt(key, &data)
            {
                return Some(plaintext.into());
            }
        }
        if let Some(encrypted_password) = Self::encrypt_password(config).as_deref() {
            tracing::warn!(
                encrypted_password,
                "raw passowrd `{password}` should be encypted"
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

    /// Returns a reference to the shared state.
    #[inline]
    pub(crate) fn shared() -> &'static State {
        LazyLock::force(&SHARED_STATE)
    }
}

impl Default for State {
    fn default() -> Self {
        SHARED_STATE.clone()
    }
}

/// Shared application state.
pub(crate) static SHARED_STATE: LazyLock<State> = LazyLock::new(|| {
    let mut app_env = "dev";
    for arg in env::args().skip(1) {
        if let Some(value) = arg.strip_prefix("--env=") {
            app_env = value.to_owned().leak();
        }
    }

    let mut state = State::new(app_env);
    state.load_config();
    state
});
