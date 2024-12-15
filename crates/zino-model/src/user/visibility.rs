use serde::{Deserialize, Serialize};
use sqlx::{Database, Decode, Type};
use strum::{AsRefStr, Display, EnumString, IntoStaticStr};
use zino_core::{BoxError, JsonValue};

/// User visibility.
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
pub enum UserVisibility {
    /// It indicates that the user can only be visible in a group.
    /// This is the default value.
    #[default]
    Internal,
    /// It indicates that the user is visible to everyone.
    Public,
    /// It indicates that the user can be visible in a group and its subgroups.
    Protected,
    /// It indicates that the user can only be visible by itself.
    Private,
}

impl From<UserVisibility> for JsonValue {
    #[inline]
    fn from(value: UserVisibility) -> Self {
        value.as_ref().into()
    }
}

impl<DB> Type<DB> for UserVisibility
where
    DB: Database,
    String: Type<DB>,
{
    #[inline]
    fn type_info() -> <DB as Database>::TypeInfo {
        <String as Type<DB>>::type_info()
    }
}

impl<'r, DB> Decode<'r, DB> for UserVisibility
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
