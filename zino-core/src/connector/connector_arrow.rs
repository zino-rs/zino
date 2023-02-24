use super::{Connector, DataSource, DataSourceConnector::Arrow};
use crate::{
    extend::{ArrowArrayExt, TomlTableExt},
    format, BoxError, Map, Record,
};
use datafusion::execution::{context::SessionContext, options::CsvReadOptions};
use std::{path::Path, sync::OnceLock};
use toml::Table;

/// A connector for Apache Arrow.
pub struct ArrowConnector {
    /// Session context.
    context: OnceLock<SessionContext>,
    /// Config.
    config: &'static Table,
}

impl ArrowConnector {
    /// Creates a new instance with the configuration.
    #[inline]
    pub fn new(config: &'static Table) -> Self {
        Self {
            context: OnceLock::new(),
            config,
        }
    }

    /// Attempts to get the session context.
    pub async fn try_get_session_context(&self) -> Result<&SessionContext, BoxError> {
        if let Some(ctx) = self.context.get() {
            return Ok(ctx);
        };

        let ctx = SessionContext::new();
        if let Some(tables) = self.config.get_array("tables") {
            for table in tables.iter().filter_map(|v| v.as_table()) {
                let table_name = table
                    .get_str("name")
                    .ok_or("the `name` field should be a str")?;
                let table_path = table
                    .get_str("path")
                    .ok_or("the `field` field should be a str")?;
                let data_type = table
                    .get_str("data-type")
                    .or_else(|| {
                        Path::new(table_path)
                            .extension()
                            .and_then(|ext| ext.to_str())
                    })
                    .ok_or("the `data-type` field should be a str")?;
                match data_type {
                    "csv" => {
                        let options = CsvReadOptions::new();
                        ctx.register_csv(table_name, table_path, options).await?;
                    }
                    _ => return Err(format!("data type `{data_type}` is unsupported").into()),
                }
            }
        }
        ctx.refresh_catalogs().await?;
        self.context.get_or_try_init(|| Ok(ctx))
    }
}

impl Connector for ArrowConnector {
    fn try_new_data_source(config: &'static Table) -> Result<DataSource, BoxError> {
        let name = config.get_str("name").unwrap_or("arrow");
        let catalog = config.get_str("catalog").unwrap_or(name);

        let connector = ArrowConnector::new(config);
        let data_source = DataSource::new(name, "arrow", catalog, Arrow(connector));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, BoxError> {
        let ctx = self.try_get_session_context().await?;
        let sql = format::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        df.collect().await?;
        Ok(None)
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, BoxError> {
        let ctx = self.try_get_session_context().await?;
        let sql = format::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        let batches = df.collect().await?;
        let mut records = Vec::new();
        let mut max_rows = 0;
        for batch in batches {
            let num_rows = batch.num_rows();
            if num_rows > max_rows {
                records.resize_with(num_rows - max_rows, Record::new);
                max_rows = num_rows;
            }
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    for i in 0..num_rows {
                        let record = &mut records[i];
                        let value = array.parse_avro_value(i)?;
                        record.push((field_name.to_owned(), value));
                    }
                }
            }
        }
        Ok(records)
    }

    async fn query_one(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<Record>, BoxError> {
        let ctx = self.try_get_session_context().await?;
        let sql = format::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        let batches = df.limit(0, Some(1))?.collect().await?;
        let mut record = Record::new();
        for batch in batches {
            for field in &batch.schema().fields {
                let field_name = field.name().as_str();
                if let Some(array) = batch.column_by_name(field_name) {
                    let value = array.parse_avro_value(0)?;
                    record.push((field_name.to_owned(), value));
                }
            }
        }
        Ok(Some(record))
    }
}
