use super::{Connector, DataSource, DataSourcePool};
use crate::extend::TomlTableExt;
use sqlx::mssql::{MssqlConnectOptions, MssqlPool, MssqlPoolOptions};
use toml::Table;

impl Connector for MssqlPool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("mssql");
        let database = config.get_str("database").unwrap_or("master");
        let connect_options = MssqlConnectOptions::new();
        let pool = MssqlPoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::Mssql(pool))
    }

    super::impl_sqlx_connector!(MssqlPool);
}
