//! Re-exports of common types in [`zino-core`].
//!
//! [`zino-core`]: https://docs.rs/zino-core

#[doc(no_inline)]
pub use zino_core::{
    application::Application,
    auth::{AccessKeyId, AuthorizationProvider, JwtClaims, UserSession},
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, JsonValueExt, TomlTableExt},
    model::{Model, ModelHooks, Mutation, Query, QueryContext},
    request::{RequestContext, Validation},
    response::{ExtractRejection, Rejection, StatusCode, WebHook},
    schedule::{AsyncCronJob, CronJob},
    state::State,
    BoxFuture, Map, Record, Uuid,
};

#[cfg(feature = "orm")]
#[doc(no_inline)]
pub use zino_core::database::{ModelAccessor, ModelHelper, Schema};
