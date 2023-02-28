use crate::{
    extend::{ArrowArrayExt, AvroRecordExt, ScalarValueExt},
    BoxError, Map, Record,
};
use apache_avro::types::Value;
use datafusion::{
    arrow::{datatypes::DataType, util},
    dataframe::DataFrame,
    error::DataFusionError,
    scalar::ScalarValue,
    variable::VarProvider,
};
use serde::de::DeserializeOwned;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use toml::Table;

/// A provider for scalar values.
#[derive(Debug, Clone)]
pub(super) struct ScalarValueProvider(HashMap<String, ScalarValue>);

impl ScalarValueProvider {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new() -> Self {
        Self(HashMap::new())
    }

    /// Reads scalar values from a TOML table.
    pub(super) fn read_toml_table(&mut self, table: &Table) {
        for (key, value) in table {
            let key = key.replace('-', "_");
            let value = ScalarValue::from_toml_value(value.to_owned());
            self.insert(key, value);
        }
    }

    /// Reads scalar values from a JSON object.
    pub(super) fn read_json_object(&mut self, map: &Map) {
        for (key, value) in map {
            let key = key.replace('-', "_");
            let value = ScalarValue::from_json_value(value.to_owned());
            self.insert(key, value);
        }
    }
}

impl Default for ScalarValueProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ScalarValueProvider {
    type Target = HashMap<String, ScalarValue>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScalarValueProvider {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VarProvider for ScalarValueProvider {
    fn get_value(&self, var_names: Vec<String>) -> Result<ScalarValue, DataFusionError> {
        var_names
            .iter()
            .find_map(|name| self.get(name.trim_start_matches('@')))
            .map(|value| value.to_owned())
            .ok_or_else(|| DataFusionError::Plan(format!("fail to get variable `{var_names:?}`")))
    }

    fn get_type(&self, var_names: &[String]) -> Option<DataType> {
        var_names.iter().find_map(|name| {
            self.get(name.trim_start_matches('@'))
                .map(|value| value.get_datatype())
        })
    }
}

/// Executor trait for [`DataFrame`](datafusion::dataframe::DataFrame).
pub trait DataFrameExecutor {
    /// Executes the `DataFrame` and returns the total number of rows affected.
    async fn execute(self) -> Result<Option<u64>, BoxError>;

    /// Executes the `DataFrame` and parses it as `Vec<Map>`.
    async fn query(self) -> Result<Vec<Record>, BoxError>;

    /// Executes the `DataFrame` and parses it as a `Map`.
    async fn query_one(self) -> Result<Option<Record>, BoxError>;

    /// Executes the `DataFrame` and parses it as `Vec<T>`.
    async fn query_as<T: DeserializeOwned>(self) -> Result<Vec<T>, BoxError>
    where
        Self: Sized,
    {
        let data = self.query().await?;
        let value = data
            .into_iter()
            .map(|record| Value::Map(record.into_avro_map()))
            .collect::<Vec<_>>();
        apache_avro::from_value(&Value::Array(value)).map_err(|err| err.into())
    }

    /// Executes the `DataFrame` and parses it as an instance of type `T`.
    async fn query_one_as<T: DeserializeOwned>(self) -> Result<Option<T>, BoxError>
    where
        Self: Sized,
    {
        if let Some(data) = self.query_one().await? {
            let value = Value::Union(1, Box::new(Value::Map(data.into_avro_map())));
            apache_avro::from_value(&value).map_err(|err| err.into())
        } else {
            Ok(None)
        }
    }

    /// Executes the `DataFrame` and creates a visual representation of record batches.
    async fn output(self) -> Result<String, BoxError>;
}

impl DataFrameExecutor for DataFrame {
    async fn execute(self) -> Result<Option<u64>, BoxError> {
        self.collect().await?;
        Ok(None)
    }

    async fn query(self) -> Result<Vec<Record>, BoxError> {
        let batches = self.collect().await?;
        let mut records = Vec::new();
        let mut max_rows = 0;
        for batch in batches {
            let num_rows = batch.num_rows();
            if num_rows > max_rows {
                records.resize_with(num_rows, Record::new);
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

    async fn query_one(self) -> Result<Option<Record>, BoxError> {
        let batches = self.limit(0, Some(1))?.collect().await?;
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

    async fn output(self) -> Result<String, BoxError> {
        let batches = self.collect().await?;
        let data = util::pretty::pretty_format_batches(&batches)?;
        Ok(data.to_string())
    }
}
