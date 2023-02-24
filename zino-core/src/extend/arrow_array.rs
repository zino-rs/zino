use crate::BoxError;
use apache_avro::types::Value;
use datafusion::arrow::{
    array::{self, Array},
    datatypes::{
        DataType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type, UInt16Type,
        UInt32Type, UInt64Type, UInt8Type,
    },
};
use std::collections::HashMap;

/// Extension trait for [`dyn Array`](datafusion::arrow::array::Array).
pub trait ArrowArrayExt {
    /// Parses an Avro value at the index.
    fn parse_avro_value(&self, index: usize) -> Result<Value, BoxError>;
}

impl ArrowArrayExt for dyn Array {
    fn parse_avro_value(&self, index: usize) -> Result<Value, BoxError> {
        if self.is_null(index) {
            return Ok(Value::Null);
        }
        let value = match self.data_type() {
            DataType::Boolean => {
                let value = array::as_boolean_array(self).value(index);
                Value::Boolean(value)
            }
            DataType::Int8 => {
                let value = array::as_primitive_array::<Int8Type>(self).value(index);
                Value::Int(value.into())
            }
            DataType::Int16 => {
                let value = array::as_primitive_array::<Int16Type>(self).value(index);
                Value::Int(value.into())
            }
            DataType::Int32 => {
                let value = array::as_primitive_array::<Int32Type>(self).value(index);
                Value::Int(value)
            }
            DataType::Int64 => {
                let value = array::as_primitive_array::<Int64Type>(self).value(index);
                Value::Long(value)
            }
            DataType::UInt8 => {
                let value = array::as_primitive_array::<UInt8Type>(self).value(index);
                Value::Int(value.into())
            }
            DataType::UInt16 => {
                let value = array::as_primitive_array::<UInt16Type>(self).value(index);
                Value::Int(value.into())
            }
            DataType::UInt32 => {
                let value = array::as_primitive_array::<UInt32Type>(self).value(index);
                Value::Int(value.try_into()?)
            }
            DataType::UInt64 => {
                let value = array::as_primitive_array::<UInt64Type>(self).value(index);
                Value::Long(value.try_into()?)
            }
            DataType::Float32 => {
                let value = array::as_primitive_array::<Float32Type>(self).value(index);
                Value::Float(value)
            }
            DataType::Float64 => {
                let value = array::as_primitive_array::<Float64Type>(self).value(index);
                Value::Double(value)
            }
            DataType::Utf8 => {
                let value = array::as_string_array(self).value(index);
                Value::String(value.to_owned())
            }
            DataType::Binary => {
                let value = array::as_generic_binary_array::<i32>(self).value(index);
                Value::Bytes(value.to_vec())
            }
            DataType::List(_field) => {
                let array = array::as_list_array(self).value(index);
                let array_length = array.len();
                let mut values = Vec::with_capacity(array_length);
                for i in 0..array_length {
                    let value = array.parse_avro_value(i)?;
                    values.push(value);
                }
                Value::Array(values)
            }
            DataType::Map(_field, _sorted) => {
                let map_array = array::as_map_array(self);
                let keys = map_array.keys();
                let values = map_array.value(index);
                let num_keys = keys.len();
                let mut hashmap = HashMap::with_capacity(num_keys);
                for i in 0..num_keys {
                    if let Value::String(key) = keys.parse_avro_value(i)? {
                        let value = values.parse_avro_value(i)?;
                        hashmap.insert(key, value);
                    } else {
                        let key_type = map_array.key_type();
                        return Err(format!("Avro map does not support `{key_type}` keys ").into());
                    }
                }
                Value::Map(hashmap)
            }
            DataType::Struct(_fields) => {
                let struct_array = array::as_struct_array(self);
                let column_names = struct_array.column_names();
                let columns = struct_array.columns();
                let num_columns = struct_array.num_columns();
                let mut hashmap = HashMap::with_capacity(num_columns);
                for i in 0..num_columns {
                    let key = column_names[i].to_owned();
                    let value = columns[i].parse_avro_value(index)?;
                    hashmap.insert(key, value);
                }
                Value::Map(hashmap)
            }
            _ => Value::Null,
        };
        Ok(value)
    }
}
