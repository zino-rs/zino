//! Utilities for formatting and parsing.

mod mask_text;
mod pdf_document;
mod str_array;

pub(crate) mod query;

pub(crate) use mask_text::mask_text;
pub(crate) use str_array::parse_str_array;

pub use pdf_document::PdfDocument;
