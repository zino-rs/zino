#![feature(iter_intersperse)]
#![feature(once_cell)]

mod database;
mod datetime;
mod request;
mod state;

/// Reexports.
pub use database::{Column, ConnectionPool, Model, Mutation, Query, Schema};
pub use datetime::DateTime;
pub use request::Validation;
pub use state::State;

/// A JSON key/value type.
pub type Map = serde_json::Map<String, serde_json::Value>;

/// A UUID is a unique 128-bit number, stored as 16 octets.
pub type Uuid = uuid::Uuid;

/// A type-erased error type.
pub type Error = Box<dyn std::error::Error + Send + Sync>;
