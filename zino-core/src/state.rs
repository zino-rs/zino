use crate::{ConnectionPool, Map};
use std::{env, fs, path::Path};
use toml::value::{Table, Value};

/// Application state.
#[derive(Debug, Clone)]
pub struct State {
    /// Environment.
    env: String,
    /// Configuration.
    config: Table,
    /// Connection pools.
    pools: Vec<ConnectionPool>,
    /// Associated data.
    data: Map,
}

impl State {
    /// Creates a new instance.
    #[inline]
    pub fn new(env: String) -> Self {
        Self {
            env,
            config: Table::new(),
            pools: Vec::new(),
            data: Map::new(),
        }
    }

    /// Loads the config file according to the specific env.
    pub fn load_config(&mut self) {
        let current_dir = env::current_dir().unwrap();
        let project_dir = Path::new(&current_dir);
        let path = if project_dir.join("./config").exists() {
            project_dir.join(format!("./config/config.{}.toml", self.env))
        } else {
            project_dir.join(format!("../config/config.{}.toml", self.env))
        };
        let config: Value = fs::read_to_string(&path)
            .unwrap_or_else(|_| panic!("fail to read config file `{:#?}`", &path))
            .parse()
            .expect("fail to parse toml value");
        match config {
            Value::Table(table) => self.config = table,
            _ => panic!("toml config file should be a table"),
        }
    }

    /// Returns the env as `&str`.
    #[inline]
    pub fn env(&self) -> &str {
        self.env.as_str()
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

    /// Returns a list of listeners as `Vec<String>`.
    pub fn listeners(&self) -> Vec<String> {
        let config = self.config();
        let mut listeners = vec![];

        // Main server.
        let main = config
            .get("main")
            .expect("the `main` field should be specified")
            .as_table()
            .expect("the `main` field should be a table");
        let main_host = main
            .get("host")
            .expect("the `main.host` field should be specified")
            .as_str()
            .expect("the `main.host` field should be a str");
        let main_port = main
            .get("port")
            .expect("the `main.port` field should be specified")
            .as_integer()
            .expect("the `main.port` field should be an integer");
        let main_listener = format!("{main_host}:{main_port}");
        listeners.push(main_listener);

        // Standbys.
        let standbys = config
            .get("standby")
            .expect("the `standby` field should be specified")
            .as_array()
            .expect("the `standby` field should be an array of tables");
        for standby in standbys {
            if standby.is_table() {
                let standby = standby
                    .as_table()
                    .expect("the `standby` field should be a table");
                let standby_host = standby
                    .get("host")
                    .expect("the `standby.host` field should be specified")
                    .as_str()
                    .expect("the `standby.host` field should be a str");
                let standby_port = standby
                    .get("port")
                    .expect("the `standby.port` field should be specified")
                    .as_integer()
                    .expect("the `standby.port` field should be an integer");
                let standby_listener = format!("{standby_host}:{standby_port}");
                listeners.push(standby_listener);
            }
        }

        listeners
    }

    /// Returns a connection pool with the specific name.
    #[inline]
    pub(crate) fn get_pool(&self, name: &str) -> Option<&ConnectionPool> {
        self.pools.iter().find(|c| c.name() == name)
    }

    /// Sets the connection pools.
    #[inline]
    pub(crate) fn set_pools(&mut self, pools: Vec<ConnectionPool>) {
        self.pools = pools;
    }
}

impl Default for State {
    #[inline]
    fn default() -> Self {
        let mut app_env = "dev".to_string();
        for arg in env::args() {
            if arg.starts_with("--env=") {
                app_env = arg.strip_prefix("--env=").unwrap().to_string();
            }
        }

        let mut state = State::new(app_env);
        state.load_config();
        state
    }
}
