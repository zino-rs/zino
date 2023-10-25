#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.png")]
#![doc(html_logo_url = "https://photino.github.io/zino-docs-zh/assets/zino-logo.svg")]

#![allow(async_fn_in_trait)]
#![allow(stable_features)]
#![forbid(unsafe_code)]

#![feature(async_fn_in_trait)]
#![feature(doc_auto_cfg)]
#![feature(lazy_cell)]
#![feature(let_chains)]

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
