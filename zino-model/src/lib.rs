//! Model types for zino.

#![feature(async_fn_in_trait)]
#![feature(once_cell)]
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

// Reexports.
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
