//! Extension traits and helper utilities.

mod avro_record;
mod header_map;
mod json_object;
mod json_value;
mod toml_table;

pub(crate) mod header;

pub use avro_record::AvroRecordExt;
pub use header_map::HeaderMapExt;
pub use json_object::JsonObjectExt;
pub use json_value::JsonValueExt;
pub use toml_table::TomlTableExt;
