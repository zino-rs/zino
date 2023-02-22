//! Utilities for formatting and parsing.

mod query;

#[cfg(any(feature = "connector", feature = "orm"))]
pub(crate) use query::format_query;
