//! Core types and traits for zino.

#![feature(async_fn_in_trait)]
#![feature(iter_intersperse)]
#![feature(once_cell)]
#![feature(string_leak)]
#![feature(type_alias_impl_trait)]

mod application;
mod authentication;
mod channel;
mod crypto;
mod database;
mod datetime;
mod request;
mod response;
mod schedule;
mod state;

// Reexports.
pub use application::Application;
pub use authentication::{AccessKeyId, Authentication, SecretAccessKey, SecurityToken};
pub use channel::{CloudEvent, Subscription};
pub use database::{Column, ConnectionPool, Model, Mutation, Query, Schema};
pub use datetime::DateTime;
pub use request::{Context, RequestContext, Validation};
pub use response::{Rejection, Response, ResponseCode};
pub use schedule::{AsyncCronJob, CronJob, Job, JobScheduler};
pub use state::State;

/// A JSON key/value type.
pub type Map = serde_json::Map<String, serde_json::Value>;

/// A UUID is a unique 128-bit number, stored as 16 octets.
pub type Uuid = uuid::Uuid;

/// An owned dynamically typed Future.
pub type BoxFuture<'a, T = ()> = futures::future::BoxFuture<'a, T>;
