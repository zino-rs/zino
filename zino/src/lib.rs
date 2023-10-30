#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.png")]
#![doc(html_logo_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]
#![allow(stable_features)]
#![forbid(unsafe_code)]
#![feature(async_fn_in_trait)]
#![feature(doc_auto_cfg)]
#![feature(extract_if)]
#![feature(lazy_cell)]
#![feature(let_chains)]

mod application;
mod channel;
mod controller;
mod endpoint;
mod middleware;
mod request;
mod response;

pub mod prelude;

pub use controller::DefaultController;

cfg_if::cfg_if! {
    if #[cfg(feature = "actix")] {
        use actix_web::{http::StatusCode, web::ServiceConfig, HttpRequest};

        use application::actix_cluster::ActixCluster;
        use request::actix_request::ActixExtractor;
        use response::actix_response::{ActixRejection, ActixResponse};

        /// HTTP server cluster for `actix-web`.
        pub type Cluster = ActixCluster;

        /// Router configure for `actix-web`.
        pub type RouterConfigure = fn(cfg: &mut ServiceConfig);

        /// A specialized request extractor for `actix-web`.
        pub type Request = ActixExtractor<HttpRequest>;

        /// A specialized response for `actix-web`.
        pub type Response = zino_core::response::Response<StatusCode>;

        /// A specialized `Result` type for `actix-web`.
        pub type Result<T = ActixResponse<StatusCode>> = std::result::Result<T, ActixRejection>;
    } else if #[cfg(feature = "axum")] {
        use axum::{body::Body, http::{self, StatusCode}};

        use application::axum_cluster::AxumCluster;
        use request::axum_request::AxumExtractor;
        use response::axum_response::{AxumRejection, AxumResponse};

        pub use channel::axum_channel::MessageChannel;

        /// HTTP server cluster for `axum`.
        pub type Cluster = AxumCluster;

        /// A specialized request extractor for `axum`.
        pub type Request = AxumExtractor<http::Request<Body>>;

        /// A specialized response for `axum`.
        pub type Response = zino_core::response::Response<StatusCode>;

        /// A specialized `Result` type for `axum`.
        pub type Result<T = AxumResponse<StatusCode>> = std::result::Result<T, AxumRejection>;
    } else if #[cfg(feature = "dioxus-desktop")] {
        use application::dioxus_desktop::DioxusDesktop;

        /// Desktop applications for `dioxus`.
        pub type Desktop<R> = DioxusDesktop<R>;
    }
}
