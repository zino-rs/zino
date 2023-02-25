use super::{
    datafusion_util::{ScalarValueProvider, TableDefinition},
    Connector, DataSource,
    DataSourceConnector::Arrow,
};
use crate::{
    application::http_client,
    extend::{ArrowArrayExt, TomlTableExt},
    format, BoxError, Map, Record,
};
use datafusion::{
    execution::{
        context::SessionContext,
        options::{AvroReadOptions, CsvReadOptions, NdJsonReadOptions, ParquetReadOptions},
    },
    variable::VarType,
};
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use toml::{Table, Value};

/// A connector for Apache Arrow.
pub struct ArrowConnector {
    /// Session context.
    context: OnceLock<SessionContext>,
    /// Root dir.
    root: PathBuf,
    /// Tables.
    tables: Option<Vec<Value>>,
    /// System variables.
    system_variables: ScalarValueProvider,
    /// User-defined variables.
    user_defined_variables: ScalarValueProvider,
}

impl ArrowConnector {
    /// Creates a new instance with the configuration.
    pub fn new(config: &Table) -> Self {
        let root = config.get_str("root").unwrap_or("./data/");
        let mut system_variables = ScalarValueProvider::default();
        if let Some(variables) = config.get_table("variables") {
            system_variables.read_toml_table(variables);
        }
        Self {
            context: OnceLock::new(),
            root: PathBuf::from(root),
            tables: config.get_array("tables").cloned(),
            system_variables,
            user_defined_variables: ScalarValueProvider::default(),
        }
    }

    /// Attempts to get the session context.
    pub async fn try_get_session_context(&self) -> Result<&SessionContext, BoxError> {
        if let Some(ctx) = self.context.get() {
            return Ok(ctx);
        };

        let ctx = SessionContext::new();
        if let Some(tables) = self.tables.as_deref() {
            let root = &self.root;
            for table in tables.iter().filter_map(|v| v.as_table()) {
                let data_type = table
                    .get_str("type")
                    .ok_or("the `type` field should be a str")?;
                let table_name = table
                    .get_str("name")
                    .ok_or("the `name` field should be a str")?;
                let table_path = if let Some(url) = table.get_str("url") {
                    let table_file_path = root.join(format!("{table_name}.{data_type}"));
                    let mut table_file = File::create(&table_file_path)?;
                    let mut res = http_client::request_builder(url, None)?.send().await?;
                    while let Some(chunk) = res.chunk().await? {
                        table_file.write_all(&chunk)?;
                    }
                    table_file_path.to_string_lossy().into_owned()
                } else {
                    table
                        .get_str("path")
                        .map(|path| root.join(path).to_string_lossy().into_owned())
                        .ok_or_else(|| format!("the path for the table `{table_name}` is absent"))?
                };
                let table_schema = table.get_schema();
                match data_type {
                    "avro" => {
                        let mut options = AvroReadOptions::default();
                        if table_schema.is_some() {
                            options.schema = table_schema.as_ref();
                        }
                        if let Some(infinite) = table.get_bool("infinite") {
                            options.infinite = infinite;
                        }
                        ctx.register_avro(table_name, &table_path, options).await?;
                    }
                    "csv" => {
                        let mut options = CsvReadOptions::default();
                        if table_schema.is_some() {
                            options.schema = table_schema.as_ref();
                        }
                        if let Some(max_records) = table.get_usize("max-records") {
                            options.schema_infer_max_records = max_records;
                        }
                        if let Some(compression_type) = table.get_compression_type() {
                            options.file_compression_type = compression_type;
                        }
                        if let Some(infinite) = table.get_bool("infinite") {
                            options.infinite = infinite;
                        }
                        ctx.register_csv(table_name, &table_path, options).await?;
                    }
                    "ndjson" => {
                        let mut options = NdJsonReadOptions::default();
                        if table_schema.is_some() {
                            options.schema = table_schema.as_ref();
                        }
                        if let Some(max_records) = table.get_usize("max-records") {
                            options.schema_infer_max_records = max_records;
                        }
                        if let Some(compression_type) = table.get_compression_type() {
                            options.file_compression_type = compression_type;
                        }
                        if let Some(infinite) = table.get_bool("infinite") {
                            options.infinite = infinite;
                        }
                        ctx.register_json(table_name, &table_path, options).await?;
                    }
                    "parquet" => {
                        let mut options = ParquetReadOptions::default();
                        if let Some(parquet_pruning) = table.get_bool("parquet-pruning") {
                            options.parquet_pruning = Some(parquet_pruning);
                        }
                        if let Some(skip_metadata) = table.get_bool("skip-metadata") {
                            options.skip_metadata = Some(skip_metadata);
                        }
                        ctx.register_parquet(table_name, &table_path, options)
                            .await?;
                    }
                    _ => return Err(format!("data type `{data_type}` is unsupported").into()),
                }
            }
        }
        ctx.register_variable(VarType::System, Arc::new(self.system_variables.clone()));
        ctx.refresh_catalogs().await?;
        self.context.get_or_try_init(|| Ok(ctx))
    }

    /// Binds system variables.
    pub async fn bind_system_variables(&mut self, variables: &Table) {
        self.system_variables.read_toml_table(variables);
        if let Ok(ctx) = self.try_get_session_context().await {
            ctx.register_variable(VarType::System, Arc::new(self.system_variables.clone()));
        }
    }

    /// Binds user defined variables.
    pub async fn bind_user_defined_variables(&mut self, variables: &Map) {
        self.user_defined_variables.read_json_object(variables);
        if let Ok(ctx) = self.try_get_session_context().await {
            ctx.register_variable(
                VarType::UserDefined,
                Arc::new(self.user_defined_variables.clone()),
            );
        }
    }
}

impl Connector for ArrowConnector {
    fn try_new_data_source(config: &Table) -> Result<DataSource, BoxError> {
        let name = config.get_str("name").unwrap_or("arrow");
        let catalog = config.get_str("catalog").unwrap_or(name);

        let connector = ArrowConnector::new(config);
        let data_source = DataSource::new("arrow", name, catalog, Arrow(connector));
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
