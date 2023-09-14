//! [![github]](https://github.com/photino/zino)
//! [![crates-io]](https://crates.io/crates/zino-model)
//! [![docs-rs]](https://docs.rs/zino-model)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs
//!
//! Domain models for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

#![doc(
    html_favicon_url = "https://user-images.githubusercontent.com/3446306/267664890-e85a1cf8-5260-4bac-b395-2341e3129e40.png"
)]
#![doc(
    html_logo_url = "https://user-images.githubusercontent.com/3446306/267670333-ac29d670-4c81-47ca-bc8c-94ec11aa28f6.svg"
)]
#![feature(async_fn_in_trait)]
#![feature(doc_auto_cfg)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![forbid(unsafe_code)]

pub mod group;
pub mod policy;
pub mod resource;
pub mod tag;
pub mod user;

pub mod application;
pub mod message;
pub mod order;

pub mod collection;
pub mod dataset;
pub mod project;
pub mod source;
pub mod task;

pub mod log;
pub mod record;

pub use group::Group;
pub use policy::Policy;
pub use resource::Resource;
pub use tag::Tag;
pub use user::User;

pub use application::Application;
pub use message::Message;
pub use order::Order;

pub use collection::Collection;
pub use dataset::Dataset;
pub use project::Project;
pub use source::Source;
pub use task::Task;

pub use log::Log;
pub use record::Record;
