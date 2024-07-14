use crate::{JsonValue, TomlValue};
use datafusion::{
    arrow::datatypes::{DataType, Field},
    scalar::ScalarValue,
};
use std::sync::Arc;

/// Extension trait for [`ScalarValue`](datafusion::scalar::ScalarValue).
pub(super) trait ScalarValueExt {
    /// Constructs an instance from a TOML value.
    fn from_toml_value(value: TomlValue) -> ScalarValue;

    /// Constructs an instance from a JSON value.
    fn from_json_value(value: JsonValue) -> ScalarValue;
}

impl ScalarValueExt for ScalarValue {
    fn from_toml_value(value: TomlValue) -> ScalarValue {
        match value {
            TomlValue::Boolean(b) => Self::Boolean(Some(b)),
            TomlValue::Integer(i) => Self::Int64(Some(i)),
            TomlValue::Float(f) => Self::Float64(Some(f)),
            TomlValue::String(s) => Self::Utf8(Some(s)),
            TomlValue::Datetime(dt) => Self::Utf8(Some(dt.to_string())),
            TomlValue::Array(vec) => {
                let mut data_type = DataType::Null;
                let scalars = vec
                    .into_iter()
                    .map(|value| {
                        let scalar = Self::from_toml_value(value);
                        if data_type == DataType::Null {
                            data_type = scalar.data_type();
                        }
                        scalar
                    })
                    .collect::<Vec<_>>();
                Self::List(Self::new_list(&scalars, &data_type, true))
            }
            TomlValue::Table(table) => {
                let entries = table
                    .into_iter()
                    .filter_map(|(key, value)| {
                        let scalar = Self::from_toml_value(value);
                        let field = Field::new(key, scalar.data_type(), true);
                        scalar.to_array().ok().map(|array| (Arc::new(field), array))
                    })
                    .collect::<Vec<_>>();
                Self::Struct(Arc::new(entries.into()))
            }
        }
    }

    /// Constructs an instance from a JSON value.
    fn from_json_value(value: JsonValue) -> ScalarValue {
        match value {
            JsonValue::Null => Self::Null,
            JsonValue::Bool(b) => Self::Boolean(Some(b)),
            JsonValue::Number(n) => {
                if let Some(i) = n.as_u64() {
                    Self::UInt64(Some(i))
                } else if let Some(i) = n.as_i64() {
                    Self::Int64(Some(i))
                } else if let Some(f) = n.as_f64() {
                    Self::Float64(Some(f))
                } else {
                    Self::Null
                }
            }
            JsonValue::String(s) => Self::Utf8(Some(s)),
            JsonValue::Array(vec) => {
                let mut data_type = DataType::Null;
                let scalars = vec
                    .into_iter()
                    .map(|value| {
                        let scalar = Self::from_json_value(value);
                        if data_type == DataType::Null {
                            data_type = scalar.data_type();
                        }
                        scalar
                    })
                    .collect::<Vec<_>>();
                Self::List(Self::new_list(&scalars, &data_type, true))
            }
            JsonValue::Object(map) => {
                let entries = map
                    .into_iter()
                    .filter_map(|(key, value)| {
                        let scalar = Self::from_json_value(value);
                        let field = Field::new(key, scalar.data_type(), true);
                        scalar.to_array().ok().map(|array| (Arc::new(field), array))
                    })
                    .collect::<Vec<_>>();
                Self::Struct(Arc::new(entries.into()))
            }
        }
    }
}
