use crate::{
    extend::{ScalarValueExt, TomlTableExt},
    BoxError, Map,
};
use datafusion::{
    arrow::datatypes::{DataType, Field, UnionMode},
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

    /// Parses the schema fields.
    fn parse_schema_fields(&self) -> Result<Vec<Field>, BoxError>;

    /// Parses the arrow data type.
    fn parse_arrow_data_type(value: &str) -> Result<DataType, BoxError>;
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

    fn parse_schema_fields(&self) -> Result<Vec<Field>, BoxError> {
        let mut fields = Vec::new();
        for (key, value) in self {
            let name = key.to_owned();
            let data_type = match value {
                Value::String(value_type) => Self::parse_arrow_data_type(value_type)?,
                Value::Array(array) => {
                    let array_length = array.len();
                    if array_length == 1 && let Some(Value::String(value_type)) = array.first() {
                        let item_data_type = Self::parse_arrow_data_type(&value_type)?;
                        let field = Field::new("item", item_data_type, true);
                        DataType::List(Box::new(field))
                    } else if array_length >= 2 {
                        let mut fields = Vec::with_capacity(array_length);
                        let mut positions = Vec::with_capacity(array_length);
                        for (index, value) in array.iter().enumerate() {
                            if let Value::String(value_type) = value {
                                let data_type = Self::parse_arrow_data_type(value_type)?;
                                let field = Field::new(index.to_string(), data_type, true);
                                fields.push(field);
                                positions.push(i8::try_from(index)?);
                            }
                        }
                        DataType::Union(fields, positions, UnionMode::Dense)
                    } else {
                        return Err(format!("schema for `{key}` should be nonempty").into());
                    }
                }
                Value::Table(table) => {
                    let fields = table.parse_schema_fields()?;
                    DataType::Struct(fields)
                }
                _ => return Err(format!("schema for `{key}` is invalid").into()),
            };
            fields.push(Field::new(name, data_type, true));
        }
        Ok(fields)
    }

    fn parse_arrow_data_type(value_type: &str) -> Result<DataType, BoxError> {
        let data_type = match value_type {
            "null" => DataType::Null,
            "boolean" => DataType::Boolean,
            "int" => DataType::Int32,
            "long" => DataType::Int64,
            "float" => DataType::Float32,
            "double" => DataType::Float64,
            "bytes" => DataType::Binary,
            "string" => DataType::Utf8,
            _ => {
                let message = format!("parsing `{value_type}` as Arrow data type is unsupported");
                return Err(message.into());
            }
        };
        Ok(data_type)
    }
}
