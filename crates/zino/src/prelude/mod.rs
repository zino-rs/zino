//! Re-exports of common types in [`zino-core`].
//!
//! [`zino-core`]: https://docs.rs/zino-core

#[doc(no_inline)]
pub use zino_core::{
    BoxFuture, Decimal, LazyLock, Map, Record, SharedString, Uuid,
    application::{Application, ApplicationCode, Plugin, StaticRecord},
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
    AccessKeyId, AuthorizationProvider, BasicCredentials, SecretAccessKey, SecurityToken,
    UserSession,
};

#[cfg(feature = "jwt")]
#[doc(no_inline)]
pub use zino_auth::JwtClaims;

#[cfg(feature = "opa")]
#[doc(no_inline)]
pub use zino_auth::RegoEngine;

#[cfg(feature = "i18n")]
#[doc(no_inline)]
pub use zino_core::{fluent_args, i18n::Intl};

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

#[cfg(feature = "orm")]
#[doc(no_inline)]
pub use zino_orm::{
    Aggregation, DerivedColumn, Entity, IntoSqlValue, JoinOn, ModelAccessor, ModelHelper,
    MutationBuilder, QueryBuilder, ScalarQuery, Schema, Transaction, Window,
};
