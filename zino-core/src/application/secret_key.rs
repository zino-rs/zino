use super::Application;
use crate::{crypto, extension::TomlTableExt};
use std::{env, sync::OnceLock};

/// Initializes the secret key.
pub(super) fn init<APP: Application + ?Sized>() {
    let checksum: [u8; 32] = APP::config()
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().first_chunk().copied())
        .unwrap_or_else(|| {
            tracing::warn!("the `checksum` is not set properly for deriving a secret key");

            let pkg_name = env::var("CARGO_PKG_NAME")
                .expect("fail to get the environment variable `CARGO_PKG_NAME`");
            let pkg_version = env::var("CARGO_PKG_VERSION")
                .expect("fail to get the environment variable `CARGO_PKG_VERSION`");
            let pkg_key = format!("{pkg_name}@{pkg_version}");
            crypto::digest(pkg_key.as_bytes())
        });

    let secret_key = crypto::derive_key("ZINO:APPLICATION", &checksum);
    SECRET_KEY
        .set(secret_key)
        .expect("fail to set the secret key");
}

/// Secret key.
pub(crate) static SECRET_KEY: OnceLock<[u8; 64]> = OnceLock::new();
