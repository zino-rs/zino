use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, IntoStaticStr};
use zino_core::JsonValue;

/// User status.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Default,
    Serialize,
    Deserialize,
    AsRefStr,
    Display,
    IntoStaticStr,
)]
#[non_exhaustive]
pub enum UserStatus {
    /// It indicates that it has not been unactivated.
    /// This is the default value.
    #[default]
    Inactive,
    /// It indicates that the user has been authenticated.
    Active,
    /// It indicates that the user has logged out.
    SignedOut,
    /// It indicates that the user has been locked and cannot be modified.
    Locked,
    /// It indicates that the user has been soft deleted.
    Deleted,
}

impl From<UserStatus> for JsonValue {
    #[inline]
    fn from(value: UserStatus) -> Self {
        value.as_ref().into()
    }
}
