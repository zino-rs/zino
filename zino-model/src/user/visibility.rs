use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, IntoStaticStr};
use zino_core::JsonValue;

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
