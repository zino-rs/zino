//! Re-exports of common types in [`zino-core`].
//!
//! [`zino-core`]: https://docs.rs/zino-core

#[doc(no_inline)]
pub use zino_core::{
    application::Application,
    auth::{AccessKeyId, JwtClaims, UserSession},
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    model::{Model, ModelHooks, Mutation, Query, QueryContext},
    request::{RequestContext, Validation},
    response::{ExtractRejection, Rejection, StatusCode},
    schedule::{AsyncCronJob, CronJob},
    state::{Data, SharedData, State},
    BoxFuture, Map, Record, Uuid,
};

#[cfg(feature = "orm")]
#[doc(no_inline)]
pub use zino_core::database::{ModelAccessor, ModelHelper, Schema};
