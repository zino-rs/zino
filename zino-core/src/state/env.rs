use self::Env::*;
use std::fmt;

/// Application running environment.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    #[default]
    /// The `dev` environment.
    Dev,
    /// The `prod` environment.
    Prod,
    /// A custom environment.
    Custom(&'static str),
}

impl Env {
    /// Returns `true` if `self` is the `dev` environment.
    #[inline]
    pub fn is_dev(&self) -> bool {
        matches!(self, Dev)
    }

    /// Returns `true` if `self` is the `prod` environment.
    #[inline]
    pub fn is_prod(&self) -> bool {
        matches!(self, Prod)
    }

    /// Returns `self` as `&'static str`.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Dev => "dev",
            Prod => "prod",
            Custom(name) => name,
        }
    }
}

impl fmt::Display for Env {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let env = self.as_str();
        write!(f, "{env}")
    }
}

impl From<&'static str> for Env {
    #[inline]
    fn from(env: &'static str) -> Self {
        match env {
            "dev" => Dev,
            "prod" => Prod,
            _ => Custom(env),
        }
    }
}
