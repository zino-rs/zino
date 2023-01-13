//! Core types and traits for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

#![feature(async_fn_in_trait)]
#![feature(iter_intersperse)]
#![feature(let_chains)]
#![feature(nonzero_min_max)]
#![feature(once_cell)]
#![feature(string_leak)]
#![feature(type_alias_impl_trait)]
#![forbid(unsafe_code)]

pub mod application;
pub mod authentication;
pub mod cache;
pub mod channel;
pub mod crypto;
pub mod database;
pub mod datetime;
pub mod request;
pub mod response;
pub mod schedule;
pub mod state;

/// A JSON key/value type.
pub type Map = serde_json::Map<String, serde_json::Value>;

/// A UUID is a unique 128-bit number, stored as 16 octets.
pub type Uuid = uuid::Uuid;

/// An allocation-optimized string.
pub type SharedString = std::borrow::Cow<'static, str>;

/// A type-erased error type.
pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;

/// An owned dynamically typed future.
pub type BoxFuture<'a, T = ()> = futures::future::BoxFuture<'a, T>;
