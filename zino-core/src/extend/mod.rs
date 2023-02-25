//! Extension traits to provide helper utilities.

#[cfg(feature = "connector-arrow")]
mod arrow_array;
#[cfg(feature = "connector-arrow")]
mod scalar_value;

mod avro_record;
mod header_map;
mod json_object;
mod toml_table;

#[cfg(feature = "connector-arrow")]
pub use arrow_array::ArrowArrayExt;
#[cfg(feature = "connector-arrow")]
pub use scalar_value::ScalarValueExt;

pub use avro_record::AvroRecordExt;
pub use header_map::HeaderMapExt;
pub use json_object::JsonObjectExt;
pub use toml_table::TomlTableExt;
