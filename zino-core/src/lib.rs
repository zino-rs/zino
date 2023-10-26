#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.png")]
#![doc(html_logo_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]
#![allow(stable_features)]
#![forbid(unsafe_code)]
#![feature(associated_type_defaults)]
#![feature(async_fn_in_trait)]
#![feature(decl_macro)]
#![feature(doc_auto_cfg)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![feature(slice_first_last_chunk)]

mod crypto;
mod encoding;
mod helper;
mod openapi;

#[cfg(feature = "accessor")]
pub mod accessor;
#[cfg(feature = "chatbot")]
pub mod chatbot;
#[cfg(feature = "connector")]
pub mod connector;
#[cfg(feature = "orm")]
pub mod orm;
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

#[doc(no_inline)]
pub use serde_json::json;

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
