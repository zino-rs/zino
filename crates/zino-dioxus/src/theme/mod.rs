//! UI themes for components.

use self::Theme::*;
use std::fmt;

/// A theme is a set of configurations controlling a component's styles and default props.
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    #[default]
    /// The `Light` theme.
    Light,
    /// The `Dark` theme.
    Dark,
    /// A custom theme.
    Custom(&'static str),
}

impl Theme {
    /// Returns `true` if it is the `Light` theme.
    #[inline]
    pub fn is_light(&self) -> bool {
        self == &Light
    }

    /// Returns `true` if it is the `Dark` theme.
    #[inline]
    pub fn is_dark(&self) -> bool {
        self == &Dark
    }

    /// Returns the theme as `str`.
    #[inline]
    pub fn as_str(&self) -> &'static str {
        match self {
            Light => "light",
            Dark => "dark",
            Custom(name) => name,
        }
    }
}

impl fmt::Display for Theme {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}
