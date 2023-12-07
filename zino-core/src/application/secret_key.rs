use super::Application;
use crate::{crypto, extension::TomlTableExt};
use std::sync::OnceLock;

/// Initializes the secret key.
pub(super) fn init<APP: Application + ?Sized>() {
    let config = APP::config();
    let checksum: [u8; 32] = config
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().first_chunk().copied())
        .unwrap_or_else(|| {
            let secret = config
                .get_str("secret")
                .map(|s| s.to_owned())
                .unwrap_or_else(|| {
                    tracing::warn!("an auto-generated `secret` is used for deriving a secret key");
                    format!("{}@{}", APP::name(), APP::version())
                });
            crypto::digest(secret.as_bytes())
        });

    let secret_key = crypto::derive_key("ZINO:APPLICATION", &checksum);
    SECRET_KEY
        .set(secret_key)
        .expect("fail to set the secret key");
}

/// Secret key.
pub(crate) static SECRET_KEY: OnceLock<[u8; 64]> = OnceLock::new();
