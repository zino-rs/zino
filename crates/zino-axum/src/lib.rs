#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]

mod application;
mod middleware;
mod request;
mod response;

pub use application::Cluster;
pub use request::Extractor;
pub use response::{AxumRejection, AxumResponse};

/// A specialized request extractor.
pub type Request = Extractor<axum::http::Request<axum::body::Body>>;

/// A specialized response.
pub type Response = zino_http::response::Response<axum::http::StatusCode>;

/// A specialized `Result` type.
pub type Result<T = AxumResponse> = std::result::Result<T, AxumRejection>;
