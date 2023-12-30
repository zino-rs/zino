#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]
#![forbid(unsafe_code)]

#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "format")]
pub mod format;
