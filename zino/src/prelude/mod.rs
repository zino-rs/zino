//! Re-exports of common types in [`zino-core`].
//!
//! [`zino-core`]: https://docs.rs/zino-core

#[doc(no_inline)]
pub use zino_core::{
    application::{Application, Plugin},
    auth::{AccessKeyId, AuthorizationProvider, SecretAccessKey, SecurityToken, UserSession},
    bail,
    datetime::{Date, DateTime, Time},
    error::Error,
    extension::{JsonObjectExt, JsonValueExt, TomlTableExt},
    file::NamedFile,
    json,
    model::{Model, ModelHooks, Mutation, Query, QueryContext},
    schedule::{AsyncCronJob, AsyncJob, AsyncJobScheduler, CronJob, Job, JobScheduler},
    state::State,
    validation::Validation,
    warn, BoxFuture, Decimal, LazyLock, Map, Record, Uuid,
};

#[cfg(feature = "i18n")]
#[doc(no_inline)]
pub use zino_core::fluent_args;

#[cfg(feature = "jwt")]
#[doc(no_inline)]
pub use zino_core::auth::JwtClaims;

#[cfg(feature = "opa")]
#[doc(no_inline)]
pub use zino_core::auth::RegoEngine;

#[cfg(feature = "orm")]
#[doc(no_inline)]
pub use zino_core::orm::{
    Aggregation, Entity, IntoSqlValue, JoinOn, ModelAccessor, ModelHelper, MutationBuilder,
    QueryBuilder, ScalarQuery, Schema, Transaction, Window,
};

#[cfg(any(feature = "actix", feature = "axum", feature = "ntex"))]
pub use zino_http::{
    reject,
    request::RequestContext,
    response::{ExtractRejection, Rejection, StatusCode, WebHook},
};
