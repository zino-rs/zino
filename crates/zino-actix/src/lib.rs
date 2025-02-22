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
pub use response::{ActixRejection, ActixResponse};

/// Router configure.
pub type RouterConfigure = fn(cfg: &mut actix_web::web::ServiceConfig);

/// A specialized request extractor.
pub type Request = request::Extractor<actix_web::HttpRequest>;

/// A specialized response.
pub type Response = zino_http::response::Response<actix_web::http::StatusCode>;

/// A specialized `Result` type.
pub type Result<T = response::ActixResponse> = std::result::Result<T, response::ActixRejection>;
