#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.png")]
#![doc(html_logo_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]
#![forbid(unsafe_code)]

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
        use crate::application::actix_cluster::ActixCluster;
        use crate::request::actix_request::ActixExtractor;
        use crate::response::actix_response::{ActixRejection, ActixResponse};
        use zino_core::response::StatusCode;

        /// HTTP server cluster for `actix-web`.
        pub type Cluster = ActixCluster;

        /// Router configure for `actix-web`.
        pub type RouterConfigure = fn(cfg: &mut actix_web::web::ServiceConfig);

        /// A specialized request extractor for `actix-web`.
        pub type Request = ActixExtractor<actix_web::HttpRequest>;

        /// A specialized response for `actix-web`.
        pub type Response = zino_core::response::Response<StatusCode>;

        /// A specialized `Result` type for `actix-web`.
        pub type Result<T = ActixResponse<StatusCode>> = std::result::Result<T, ActixRejection>;
    } else if #[cfg(feature = "axum")] {
        use crate::application::axum_cluster::AxumCluster;
        use crate::request::axum_request::AxumExtractor;
        use crate::response::axum_response::{AxumRejection, AxumResponse};
        use zino_core::response::StatusCode;

        pub use channel::axum_channel::MessageChannel;

        /// HTTP server cluster for `axum`.
        pub type Cluster = AxumCluster;

        /// A specialized request extractor for `axum`.
        pub type Request = AxumExtractor<axum::http::Request<axum::body::Body>>;

        /// A specialized response for `axum`.
        pub type Response = zino_core::response::Response<StatusCode>;

        /// A specialized `Result` type for `axum`.
        pub type Result<T = AxumResponse<StatusCode>> = std::result::Result<T, AxumRejection>;
    } else if #[cfg(feature = "dioxus-desktop")] {
        use crate::application::dioxus_desktop::DioxusDesktop;

        /// Desktop applications for `dioxus`.
        pub type Desktop<R> = DioxusDesktop<R>;
    }
}
