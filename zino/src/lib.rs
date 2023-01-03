//! A minimal MVC framework.

#![feature(async_fn_in_trait)]
#![feature(once_cell)]

mod channel;
mod cluster;
mod endpoint;
mod middleware;
mod request;

#[cfg(feature = "axum-server")]
pub use cluster::axum_cluster::AxumCluster;

#[cfg(feature = "axum-server")]
pub use request::axum_request::AxumExtractor;

#[cfg(feature = "axum-server")]
/// A specialized request extractor for axum.
pub type Request = AxumExtractor<axum::http::Request<axum::body::Body>>;

#[cfg(feature = "axum-server")]
/// A specialized `Result` type for axum.
pub type Result<T = axum::http::Response<axum::body::Full<axum::body::Bytes>>> =
    std::result::Result<T, T>;
