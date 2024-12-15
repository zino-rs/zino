//! Wrappers for manipulating common file formats.
//!
//! ## Feature flags
//!
//! The following optional features are available:
//!
//! | Name          | Description                                          | Default? |
//! |---------------|------------------------------------------------------|----------|
//! | `format-pdf`  | Enables the support for `PDF` documents.             | No       |

#[cfg(feature = "format-pdf")]
mod pdf_document;

#[cfg(feature = "format-pdf")]
pub use pdf_document::PdfDocument;
