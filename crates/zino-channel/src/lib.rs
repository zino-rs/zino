#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![doc(html_favicon_url = "https://zino.cc/assets/zino-logo.png")]
#![doc(html_logo_url = "https://zino.cc/assets/zino-logo.svg")]

mod cloud_event;
mod subscription;

pub use cloud_event::CloudEvent;
pub use subscription::Subscription;

#[cfg(feature = "flume")]
mod flume;

#[cfg(feature = "flume")]
pub use flume::MessageChannel;
