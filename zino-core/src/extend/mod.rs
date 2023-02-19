//! Extension traits to provide helper utilities.

#[cfg(any(feature = "connector", feature = "orm"))]
mod avro_record;

mod header_map;
mod json_object;
mod toml_table;

#[cfg(any(feature = "connector", feature = "orm"))]
pub use avro_record::AvroRecordExt;

pub use header_map::HeaderMapExt;
pub use json_object::JsonObjectExt;
pub use toml_table::TomlTableExt;
