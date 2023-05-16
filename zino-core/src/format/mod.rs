//! Utilities for formatting and parsing.

pub(crate) mod mask_text;
pub(crate) mod str_array;

pub(crate) use mask_text::mask_text;

#[cfg(any(feature = "connector", feature = "orm"))]
mod query;

#[cfg(any(feature = "connector", feature = "orm"))]
pub(crate) use query::format_query;
