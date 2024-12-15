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

impl fmt::Display for Theme {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let theme = match self {
            Light => "light",
            Dark => "dark",
            Custom(name) => name,
        };
        write!(f, "{theme}")
    }
}
