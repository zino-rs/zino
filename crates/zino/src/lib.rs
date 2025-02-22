#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]

mod controller;

pub mod prelude;

pub use controller::DefaultController;

cfg_if::cfg_if! {
    if #[cfg(feature = "actix")] {
        #[doc(no_inline)]
        pub use zino_actix::{Cluster, Request, Response, Result, RouterConfigure};
    } else if #[cfg(feature = "axum")] {
        #[doc(no_inline)]
        pub use zino_axum::{Cluster, Request, Response, Result};
    } else if #[cfg(feature = "ntex")] {
        #[doc(no_inline)]
        pub use zino_ntex::{Cluster, Request, Response, Result, RouterConfigure};
    }
}

#[cfg(feature = "dioxus-desktop")]
#[doc(no_inline)]
pub use zino_dioxus::application::Desktop;
