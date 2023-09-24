//! [![github]](https://github.com/photino/zino)
//! [![crates-io]](https://crates.io/crates/zino-core)
//! [![docs-rs]](https://docs.rs/zino-core)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs
//!
//! Core types and traits for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

#![doc(
    html_favicon_url = "https://user-images.githubusercontent.com/3446306/267664890-e85a1cf8-5260-4bac-b395-2341e3129e40.png"
)]
#![doc(
    html_logo_url = "https://user-images.githubusercontent.com/3446306/267670333-ac29d670-4c81-47ca-bc8c-94ec11aa28f6.svg"
)]
#![feature(associated_type_defaults)]
#![feature(async_fn_in_trait)]
#![feature(decl_macro)]
#![feature(doc_auto_cfg)]
#![feature(iter_intersperse)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![feature(result_option_inspect)]
#![feature(slice_first_last_chunk)]
#![forbid(unsafe_code)]

mod crypto;
mod encoding;
mod helper;
mod openapi;

#[cfg(feature = "accessor")]
pub mod accessor;
#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "chatbot")]
pub mod chatbot;
#[cfg(feature = "connector")]
pub mod connector;
#[cfg(feature = "orm")]
pub mod database;
#[cfg(feature = "format")]
pub mod format;
#[cfg(feature = "view")]
pub mod view;

pub mod application;
pub mod auth;
pub mod channel;
pub mod datetime;
pub mod error;
pub mod extension;
pub mod file;
pub mod i18n;
pub mod model;
pub mod request;
pub mod response;
pub mod schedule;
pub mod state;
pub mod trace;

/// A JSON value.
pub type JsonValue = serde_json::Value;

/// A JSON key-value type.
pub type Map = serde_json::Map<String, JsonValue>;

/// An Avro value.
pub type AvroValue = apache_avro::types::Value;

/// A schema-less Avro record value.
pub type Record = Vec<(String, AvroValue)>;

/// A TOML value.
pub type TomlValue = toml::Value;

/// A Universally Unique Identifier (UUID).
pub type Uuid = uuid::Uuid;

/// An allocation-optimized string.
pub type SharedString = std::borrow::Cow<'static, str>;

/// An owned dynamically typed error.
pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;

/// An owned dynamically typed future.
pub type BoxFuture<'a, T = ()> =
    std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
