use self::ServerTag::*;
use std::fmt;

/// A server tag is used to distinguish different servers.
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum ServerTag {
    #[default]
    /// The `main` server.
    Main,
    /// The `debug` server.
    Debug,
    /// The `standby` server with a custom tag.
    Standby(String),
}

impl ServerTag {
    /// Returns `true` if `self` is the `main` server.
    #[inline]
    pub fn is_main(&self) -> bool {
        matches!(self, Main)
    }

    /// Returns `true` if `self` is the `debug` server.
    #[inline]
    pub fn is_debug(&self) -> bool {
        matches!(self, Debug)
    }

    /// Returns `true` if `self` is the `standby` server.
    #[inline]
    pub fn is_standby(&self) -> bool {
        matches!(self, Standby(_))
    }

    /// Returns `self` as `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Main => "main",
            Debug => "debug",
            Standby(tag) => tag.as_str(),
        }
    }
}

impl fmt::Display for ServerTag {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tag = self.as_str();
        write!(f, "{tag}")
    }
}

impl From<&str> for ServerTag {
    #[inline]
    fn from(tag: &str) -> Self {
        match tag {
            "main" => Main,
            "debug" => Debug,
            _ => Standby(tag.to_owned()),
        }
    }
}
