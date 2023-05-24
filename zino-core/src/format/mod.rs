//! Utilities for formatting and parsing.

mod mask_text;
mod str_array;

#[cfg(any(feature = "connector", feature = "orm"))]
pub(crate) mod query;

pub(crate) use mask_text::mask_text;
pub(crate) use str_array::parse_str_array;
