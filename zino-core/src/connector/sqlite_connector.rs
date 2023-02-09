use super::{Connector, DataSource, DataSourcePool};
use crate::extend::TomlTableExt;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use toml::Table;

impl Connector for SqlitePool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("sqlite");
        let database = config.get_str("database").unwrap_or_default();
        let connect_options = SqliteConnectOptions::new();
        let pool = SqlitePoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::Sqlite(pool))
    }

    super::impl_sqlx_connector!(SqlitePool);
}
