#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]

mod file;

#[cfg(feature = "accessor")]
mod accessor;

pub use file::NamedFile;

#[cfg(feature = "accessor")]
pub use accessor::GlobalAccessor;
