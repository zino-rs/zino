//! Utilities for formatting and parsing.

pub(crate) mod base64;
pub(crate) mod str_array;

#[cfg(any(feature = "connector", feature = "orm"))]
mod query;

#[cfg(any(feature = "connector", feature = "orm"))]
pub(crate) use query::format_query;
