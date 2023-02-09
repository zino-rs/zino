use super::{Connector, DataSource, DataSourcePool};
use crate::extend::TomlTableExt;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool, MySqlPoolOptions};
use toml::Table;

impl Connector for MySqlPool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("mysql");
        let database = config.get_str("database").unwrap_or_default();
        let connect_options = MySqlConnectOptions::new();
        let pool = MySqlPoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::MySql(pool))
    }

    super::impl_sqlx_connector!(MySqlPool);
}
