//! [`zino`] is a full-featured web application framework for Rust
//! with a focus on productivity and performance.
//!
//! ## Highlights
//!
//! - 🚀 Out-of-the-box features for rapid application development.
//! - ✨ Minimal design, modular architecture and high-level abstractions.
//! - ⚡ Embrace practical conventions to get the best performance.
//! - 🐘 Highly optimized ORM for PostgreSQL built on top of [`sqlx`].
//! - 🕗 Lightweight scheduler for sync and async cron jobs.
//! - 📊 Support for [`tracing`], [`metrics`] and logging.
//!
//! ## Getting started
//!
//! You can start with the example [`axum-app`].
//!
//! ## Feature flags
//!
//! Currently, we only provide the `axum` feature to enable an integration with [`axum`].
//!
//! [`zino`]: https://github.com/photino/zino
//! [`sqlx`]: https://crates.io/crates/sqlx
//! [`tracing`]: https://crates.io/crates/tracing
//! [`metrics`]: https://crates.io/crates/metrics
//! [`axum`]: https://crates.io/crates/axum
//! [`axum-app`]: https://github.com/photino/zino/tree/main/examples/axum-app

#![feature(async_fn_in_trait)]
#![feature(doc_auto_cfg)]
#![feature(once_cell)]
#![feature(result_option_inspect)]
#![feature(string_leak)]
#![forbid(unsafe_code)]

mod channel;
mod cluster;
mod endpoint;
mod middleware;
mod request;

#[doc(no_inline)]
pub use zino_core::{
    application::Application,
    database::Schema,
    datetime::DateTime,
    extend::JsonObjectExt,
    model::{Model, Query},
    request::RequestContext,
    response::ExtractRejection,
    schedule::{AsyncCronJob, CronJob},
    BoxFuture, Map, Record, Uuid,
};

#[cfg(feature = "axum")]
pub use cluster::axum_cluster::AxumCluster;
#[cfg(feature = "axum")]
pub use request::axum_request::AxumExtractor;

#[cfg(feature = "axum")]
/// A specialized request extractor for `axum`.
pub type Request = AxumExtractor<axum::http::Request<axum::body::Body>>;

#[cfg(feature = "axum")]
/// A specialized response for `axum`.
pub type Response = zino_core::response::Response<axum::http::StatusCode>;

#[cfg(feature = "axum")]
/// A specialized `Result` type for `axum`.
pub type Result<T = axum::http::Response<axum::body::Full<axum::body::Bytes>>> =
    std::result::Result<T, T>;
