//! Core types and traits for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

#![feature(async_fn_in_trait)]
#![feature(decl_macro)]
#![feature(doc_auto_cfg)]
#![feature(is_some_and)]
#![feature(iter_intersperse)]
#![feature(let_chains)]
#![feature(nonzero_min_max)]
#![feature(once_cell)]
#![feature(result_option_inspect)]
#![feature(string_leak)]
#![forbid(unsafe_code)]

mod crypto;
mod format;

#[cfg(feature = "accessor")]
pub mod accessor;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "connector")]
pub mod connector;
#[cfg(feature = "orm")]
pub mod database;
#[cfg(feature = "view")]
pub mod view;

pub mod application;
pub mod authentication;
pub mod channel;
pub mod datetime;
pub mod extend;
pub mod i18n;
pub mod model;
pub mod request;
pub mod response;
pub mod schedule;
pub mod state;
pub mod trace;

/// A JSON key/value type.
pub type Map = serde_json::Map<String, serde_json::Value>;

/// A schema-less Avro record value.
pub type Record = Vec<(String, apache_avro::types::Value)>;

/// A Universally Unique Identifier (UUID).
pub type Uuid = uuid::Uuid;

/// An allocation-optimized string.
pub type SharedString = std::borrow::Cow<'static, str>;

/// A type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;

/// An owned dynamically typed future.
pub type BoxFuture<'a, T = ()> =
    std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
