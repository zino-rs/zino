//! [![github]](https://github.com/photino/zino)
//! [![crates-io]](https://crates.io/crates/zino-model)
//! [![docs-rs]](https://docs.rs/zino-model)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?labelColor=555555&logo=docs.rs
//!
//! Model types for [`zino`].
//!
//! [`zino`]: https://github.com/photino/zino

#![feature(async_fn_in_trait)]
#![feature(lazy_cell)]
#![feature(let_chains)]
#![forbid(unsafe_code)]

mod group;
mod policy;
mod resource;
mod tag;
mod user;

mod message;
mod order;

mod collection;
mod dataset;
mod source;
mod task;

mod log;
mod record;

pub use group::Group;
pub use policy::Policy;
pub use resource::Resource;
pub use tag::Tag;
pub use user::User;

pub use message::Message;
pub use order::Order;

pub use collection::Collection;
pub use dataset::Dataset;
pub use source::Source;
pub use task::Task;

pub use log::Log;
pub use record::Record;
