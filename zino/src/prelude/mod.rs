//! Re-exports of common types in [`zino-core`].
//!
//! [`zino-core`]: https://docs.rs/zino-core

#[doc(no_inline)]
pub use zino_core::{
    application::Application,
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    model::{Model, ModelAccessor, Mutation, Query},
    request::{RequestContext, Validation},
    response::{ExtractRejection, Rejection},
    schedule::{AsyncCronJob, CronJob},
    state::State,
    BoxFuture, Map, Record, Uuid,
};

#[cfg(feature = "orm")]
#[doc(no_inline)]
pub use zino_core::database::Schema;
