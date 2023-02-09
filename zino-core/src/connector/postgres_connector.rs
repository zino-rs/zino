use super::{Connector, DataSource, DataSourcePool};
use crate::extend::TomlTableExt;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use toml::Table;

impl Connector for PgPool {
    fn new_data_source(config: &'static Table) -> DataSource {
        let name = config.get_str("name").unwrap_or("postgres");
        let database = config.get_str("database").unwrap_or("postgres");
        let connect_options = PgConnectOptions::new();
        let pool = PgPoolOptions::new().connect_lazy_with(connect_options);
        DataSource::new(name, database, DataSourcePool::Postgres(pool))
    }

    super::impl_sqlx_connector!(PgPool);
}
