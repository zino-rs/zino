//! Utilities for DataFusion.

use super::{Connector, DataSource, DataSourceConnector::Arrow};
use crate::{
    application::{http_client, PROJECT_DIR},
    bail,
    error::Error,
    extension::TomlTableExt,
    helper, warn, LazyLock, Map, Record,
};
use datafusion::{
    arrow::{datatypes::Schema, record_batch::RecordBatch},
    dataframe::DataFrame,
    datasource::file_format::file_compression_type::FileCompressionType,
    execution::{
        context::{SessionConfig, SessionContext},
        options::{AvroReadOptions, CsvReadOptions, NdJsonReadOptions, ParquetReadOptions},
        runtime_env::RuntimeEnv,
        session_state::{SessionState, SessionStateBuilder},
    },
    variable::VarType,
};
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    sync::{Arc, OnceLock},
};
use toml::value::{Array, Table};

mod arrow_array;
mod arrow_field;
mod arrow_schema;
mod data_frame;
mod scalar_provider;
mod scalar_value;

pub use data_frame::DataFrameExecutor;

use arrow_array::ArrowArrayExt;
use arrow_field::ArrowFieldExt;
use arrow_schema::ArrowSchemaExt;
use scalar_provider::ScalarValueProvider;
use scalar_value::ScalarValueExt;

/// A connector for Apache Arrow.
pub struct ArrowConnector {
    /// Session context.
    context: OnceLock<SessionContext>,
    /// Root dir.
    root: PathBuf,
    /// Tables.
    tables: Option<Array>,
    /// System variables.
    system_variables: ScalarValueProvider,
    /// User-defined variables.
    user_defined_variables: ScalarValueProvider,
}

impl ArrowConnector {
    /// Creates a new instance with the default configuration.
    #[inline]
    pub fn new() -> Self {
        Self {
            context: OnceLock::new(),
            root: PROJECT_DIR.join("local/data/"),
            tables: None,
            system_variables: ScalarValueProvider::default(),
            user_defined_variables: ScalarValueProvider::default(),
        }
    }

    /// Creates a new instance with the configuration.
    pub fn with_config(config: &Table) -> Self {
        let root = config.get_str("root").unwrap_or("local/data/");
        let mut system_variables = ScalarValueProvider::default();
        if let Some(variables) = config.get_table("variables") {
            system_variables.read_toml_table(variables);
        }
        Self {
            context: OnceLock::new(),
            root: PROJECT_DIR.join(root),
            tables: config.get_array("tables").cloned(),
            system_variables,
            user_defined_variables: ScalarValueProvider::default(),
        }
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

    /// Attempts to get the session context.
    pub async fn try_get_session_context(&self) -> Result<&SessionContext, Error> {
        if let Some(ctx) = self.context.get() {
            return Ok(ctx);
        };

        let ctx = SessionContext::new_with_state(SHARED_SESSION_STATE.clone());
        if let Some(tables) = self.tables.as_deref() {
            let root = &self.root;
            for table in tables.iter().filter_map(|v| v.as_table()) {
                let data_type = table
                    .get_str("type")
                    .ok_or_else(|| warn!("the `type` field should be a str"))?;
                let table_name = table
                    .get_str("name")
                    .ok_or_else(|| warn!("the `name` field should be a str"))?;
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
                        .ok_or_else(|| warn!("the path for the table `{}` is absent", table_name))?
                };
                let table_schema = if let Some(schema) = table.get_table("schema") {
                    Some(Schema::try_from_toml_table(schema)?)
                } else {
                    None
                };
                match data_type {
                    "avro" => {
                        let mut options = AvroReadOptions::default();
                        if table_schema.is_some() {
                            options.schema = table_schema.as_ref();
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
                        if let Some(compression_type) = table.get_str("compression-type") {
                            options.file_compression_type = match compression_type {
                                "bzip2" => FileCompressionType::BZIP2,
                                "gzip" => FileCompressionType::GZIP,
                                "xz" => FileCompressionType::XZ,
                                _ => FileCompressionType::UNCOMPRESSED,
                            };
                        }
                        ctx.register_csv(table_name, &table_path, options).await?;
                    }
                    "ndjson" => {
                        let mut options = NdJsonReadOptions::default().file_extension(".ndjson");
                        if table_schema.is_some() {
                            options.schema = table_schema.as_ref();
                        }
                        if let Some(max_records) = table.get_usize("max-records") {
                            options.schema_infer_max_records = max_records;
                        }
                        if let Some(compression_type) = table.get_str("compression-type") {
                            options.file_compression_type = match compression_type {
                                "bzip2" => FileCompressionType::BZIP2,
                                "gzip" => FileCompressionType::GZIP,
                                "xz" => FileCompressionType::XZ,
                                _ => FileCompressionType::UNCOMPRESSED,
                            };
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
                    _ => {
                        bail!("data type `{}` is unsupported", data_type);
                    }
                }
            }
        }
        ctx.register_variable(VarType::System, Arc::new(self.system_variables.clone()));
        ctx.refresh_catalogs().await?;
        Ok(self.context.get_or_init(|| ctx))
    }

    /// Attempts to create a [`DateFrame`](datafusion::dataframe::DataFrame)
    /// from reading Avro records.
    pub async fn read_avro_records(&self, records: &[Record]) -> Result<DataFrame, Error> {
        let ctx = self.try_get_session_context().await?;
        let schema = if let Some(record) = records.first() {
            Schema::try_from_avro_record(record)?
        } else {
            Schema::empty()
        };

        let columns = schema.collect_columns_from_avro_records(records);
        let batch = RecordBatch::try_new(Arc::new(schema), columns)?;
        ctx.read_batch(batch).map_err(Error::from)
    }
}

impl Default for ArrowConnector {
    fn default() -> Self {
        Self::new()
    }
}

impl Connector for ArrowConnector {
    fn try_new_data_source(config: &Table) -> Result<DataSource, Error> {
        let name = config.get_str("name").unwrap_or("arrow");
        let catalog = config.get_str("catalog").unwrap_or(name);

        let connector = ArrowConnector::with_config(config);
        let data_source = DataSource::new("arrow", None, name, catalog, Arrow(connector));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error> {
        let ctx = self.try_get_session_context().await?;
        let sql = helper::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        df.execute().await
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, Error> {
        let ctx = self.try_get_session_context().await?;
        let sql = helper::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        df.query().await
    }

    async fn query_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let ctx = self.try_get_session_context().await?;
        let sql = helper::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        df.query_as().await
    }

    async fn query_one(&self, query: &str, params: Option<&Map>) -> Result<Option<Record>, Error> {
        let ctx = self.try_get_session_context().await?;
        let sql = helper::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        df.query_one().await
    }

    async fn query_one_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        let ctx = self.try_get_session_context().await?;
        let sql = helper::format_query(query, params);
        let df = ctx.sql(&sql).await?;
        df.query_one_as().await
    }
}

/// Shared session state for DataFusion.
static SHARED_SESSION_STATE: LazyLock<SessionState> = LazyLock::new(|| {
    SessionStateBuilder::new()
        .with_config(SessionConfig::new())
        .with_runtime_env(Arc::new(RuntimeEnv::default()))
        .with_default_features()
        .build()
});
