//! Wrappers for manipulating common file formats.

#[cfg(feature = "format-pdf")]
mod pdf_document;

#[cfg(feature = "format-pdf")]
pub use pdf_document::PdfDocument;
