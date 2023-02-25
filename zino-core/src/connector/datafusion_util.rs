use crate::{
    extend::{ScalarValueExt, TomlTableExt},
    Map,
};
use datafusion::{
    arrow::datatypes::{DataType, Field, Schema},
    datasource::file_format::file_type::FileCompressionType,
    error::DataFusionError,
    scalar::ScalarValue,
    variable::VarProvider,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use toml::{Table, Value};

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

/// Trait for table definition.
pub(super) trait TableDefinition {
    /// Gets the file compression type.
    fn get_compression_type(&self) -> Option<FileCompressionType>;

    /// Gets the schema.
    fn get_schema(&self) -> Option<Schema>;
}

impl TableDefinition for Table {
    fn get_compression_type(&self) -> Option<FileCompressionType> {
        self.get_str("compression-type")
            .map(|compression_type| match compression_type {
                "bzip2" => FileCompressionType::BZIP2,
                "gzip" => FileCompressionType::GZIP,
                "xz" => FileCompressionType::XZ,
                _ => FileCompressionType::UNCOMPRESSED,
            })
    }

    fn get_schema(&self) -> Option<Schema> {
        self.get_table("schema").map(|schema| {
            let mut fields = Vec::new();
            for (key, value) in schema {
                let name = key.to_owned();
                let data_type = match value {
                    Value::String(s) => match s.as_str() {
                        "string" => DataType::Utf8,
                        _ => DataType::Null,
                    },
                    _ => DataType::Null,
                };
                fields.push(Field::new(name, data_type, true));
            }
            Schema::new(fields)
        })
    }
}
