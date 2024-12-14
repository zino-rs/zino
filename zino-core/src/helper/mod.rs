/// Helper utilities.
mod header;
mod mask_text;
mod str_array;

pub(crate) use header::check_json_content_type;
pub(crate) use mask_text::mask_text;
pub(crate) use str_array::parse_str_array;

#[cfg(any(feature = "orm", feature = "orm-mysql", feature = "orm-postgres",))]
pub(crate) mod query;

#[cfg(any(feature = "orm", feature = "orm-mysql", feature = "orm-postgres",))]
mod sql_query;

#[cfg(any(feature = "orm", feature = "orm-mysql", feature = "orm-postgres",))]
pub(crate) use query::format_query;

#[cfg(any(feature = "orm", feature = "orm-mysql", feature = "orm-postgres",))]
pub(crate) use sql_query::prepare_sql_query;
