use super::{Connector, DataSource, DataSourceConnector::Mssql};
use crate::{extend::TomlTableExt, state::State, BoxError};
use sqlx::mssql::{MssqlPool, MssqlPoolOptions};
use std::time::Duration;
use toml::Table;

impl Connector for MssqlPool {
    fn try_new_data_source(config: &Table) -> Result<DataSource, BoxError> {
        let name = config.get_str("name").unwrap_or("mssql");
        let database = config.get_str("database").unwrap_or("master");
        let authority = State::format_authority(config, Some(1433));
        let dsn = format!("mssql://{authority}/{database}");

        let max_connections = config.get_u32("max-connections").unwrap_or(16);
        let min_connections = config.get_u32("min-connections").unwrap_or(2);
        let max_lifetime = config
            .get_duration("max-lifetime")
            .unwrap_or_else(|| Duration::from_secs(60 * 60));
        let idle_timeout = config
            .get_duration("idle-timeout")
            .unwrap_or_else(|| Duration::from_secs(10 * 60));
        let acquire_timeout = config
            .get_duration("acquire-timeout")
            .unwrap_or_else(|| Duration::from_secs(30));
        let pool_options = MssqlPoolOptions::new()
            .max_connections(max_connections)
            .min_connections(min_connections)
            .max_lifetime(max_lifetime)
            .idle_timeout(idle_timeout)
            .acquire_timeout(acquire_timeout);
        let pool = pool_options.connect_lazy(&dsn)?;
        let data_source = DataSource::new("mssql", None, name, database, Mssql(pool));
        Ok(data_source)
    }

    super::sqlx_common::impl_sqlx_connector!(MssqlPool);
}
