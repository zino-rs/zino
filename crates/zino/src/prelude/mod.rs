//! Re-exports of common types in [`zino-core`].
//!
//! [`zino-core`]: https://docs.rs/zino-core

#[doc(no_inline)]
pub use zino_core::{
    BoxFuture, Decimal, LazyLock, Map, Record, Uuid,
    application::{Application, Plugin},
    bail,
    datetime::{Date, DateTime, Time},
    error::Error,
    extension::{JsonObjectExt, JsonValueExt, TomlTableExt},
    json,
    model::{Model, ModelHooks, Mutation, Query, QueryContext},
    schedule::{AsyncCronJob, AsyncJob, AsyncJobScheduler, CronJob, Job, JobContext, JobScheduler},
    state::State,
    validation::Validation,
    warn,
};

#[doc(no_inline)]
pub use zino_storage::NamedFile;

#[cfg(feature = "auth")]
#[doc(no_inline)]
pub use zino_auth::{
    AccessKeyId, AuthorizationProvider, SecretAccessKey, SecurityToken, UserSession,
};

#[cfg(feature = "i18n")]
#[doc(no_inline)]
pub use zino_http::fluent_args;

#[cfg(feature = "jwt")]
#[doc(no_inline)]
pub use zino_auth::JwtClaims;

#[cfg(feature = "opa")]
#[doc(no_inline)]
pub use zino_auth::RegoEngine;

#[cfg(feature = "orm")]
#[doc(no_inline)]
pub use zino_orm::{
    Aggregation, DerivedColumn, Entity, IntoSqlValue, JoinOn, ModelAccessor, ModelHelper,
    MutationBuilder, QueryBuilder, ScalarQuery, Schema, Transaction, Window,
};

#[cfg(feature = "preferences")]
#[doc(no_inline)]
pub use zino_core::application::Preferences;

#[cfg(any(feature = "actix", feature = "axum", feature = "ntex"))]
#[doc(no_inline)]
pub use zino_http::{
    reject,
    request::RequestContext,
    response::{ExtractRejection, Rejection, StatusCode, WebHook},
};

#[cfg(feature = "inertia")]
#[doc(no_inline)]
pub use zino_http::inertia::InertiaPage;
