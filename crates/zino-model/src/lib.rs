#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]
#![allow(async_fn_in_trait)]

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

pub use group::{Group, GroupColumn};
pub use policy::{Policy, PolicyColumn};
pub use resource::{Resource, ResourceColumn};
pub use tag::{Tag, TagColumn};
pub use user::{User, UserColumn};

pub use application::{Application, ApplicationColumn};
pub use message::{Message, MessageColumn};
pub use order::{Order, OrderColumn};

pub use collection::{Collection, CollectionColumn};
pub use dataset::{Dataset, DatasetColumn};
pub use project::{Project, ProjectColumn};
pub use source::{Source, SourceColumn};
pub use task::{Task, TaskColumn};

pub use log::{Log, LogColumn};
pub use record::{Record, RecordColumn};
