//! [`zino`] is a full featured web application framework for Rust
//! which focuses on productivity and performance.
//!
//! ## Highlights
//!
//! - 🚀 Out-of-the-box features for rapid application development.
//! - ✨ Minimal design, modular architecture and high-level abstractions.
//! - ⚡ Embrace practical conventions to get the best performance.
//! - 🐘 Highly optimized ORM for PostgreSQL built with [`sqlx`].
//! - ⏲ Lightweight scheduler for sync and async cron jobs.
//! - 📊 Support for `logging`, [`tracing`] and [`metrics`].
//!
//! ## Getting started
//!
//! You can start with the example [`axum-app`].
//!
//! [`zino`]: https://github.com/photino/zino
//! [`sqlx`]: https://crates.io/crates/sqlx
//! [`tracing`]: https://crates.io/crates/tracing
//! [`metrics`]: https://crates.io/crates/metrics
//! [`axum-app`]: https://github.com/photino/zino/tree/main/examples/axum-app

#![feature(async_fn_in_trait)]
#![feature(once_cell)]
#![feature(result_option_inspect)]
#![feature(string_leak)]
#![forbid(unsafe_code)]

mod channel;
mod cluster;
mod endpoint;
mod middleware;
mod request;

cfg_if::cfg_if! {
    if #[cfg(feature = "axum-server")] {
        // Reexports.
        pub use cluster::axum_cluster::AxumCluster;
        pub use request::axum_request::AxumExtractor;

        /// A specialized request extractor for `axum`.
        pub type Request = AxumExtractor<axum::http::Request<axum::body::Body>>;

        /// A specialized `Result` type for `axum`.
        pub type Result<T = axum::http::Response<axum::body::Full<axum::body::Bytes>>> =
            std::result::Result<T, T>;
    }
}
