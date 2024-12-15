use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode, Type};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};
use zino_core::{BoxError, JsonValue};

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
    EnumString,
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

impl<DB> Type<DB> for UserStatus
where
    DB: Database,
    String: Type<DB>,
{
    #[inline]
    fn type_info() -> <DB as Database>::TypeInfo {
        <String as Type<DB>>::type_info()
    }
}

impl<'r, DB> Decode<'r, DB> for UserStatus
where
    DB: Database,
    &'r str: Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxError> {
        let value = <&'r str as Decode<'r, DB>>::decode(value)?;
        Ok(value.parse()?)
    }
}
