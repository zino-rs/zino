//! Application integrations.

#[cfg(feature = "desktop")]
mod desktop;

#[cfg(feature = "desktop")]
mod preferences;

#[cfg(feature = "desktop")]
pub use desktop::Desktop;

#[cfg(feature = "desktop")]
pub use preferences::Preferences;
