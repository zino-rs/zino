use super::Application;
use crate::extend::TomlTableExt;
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use std::{env, sync::OnceLock};

/// Initializes the secret key.
pub(super) fn init<APP: Application + ?Sized>() {
    let app_checksum: [u8; 32] = APP::config()
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().try_into().ok())
        .unwrap_or_else(|| {
            let pkg_name = env::var("CARGO_PKG_NAME").expect("failed to get crate name");
            let pkg_version = env::var("CARGO_PKG_VERSION").expect("failed to get crate version");
            let pkg_description = env::var("CARGO_PKG_DESCRIPTION").unwrap_or_default();
            let pkg_key = format!("{pkg_name}@{pkg_version}:{pkg_description}");
            let mut hasher = Sha256::new();
            hasher.update(pkg_key.as_bytes());
            hasher.finalize().into()
        });

    let mut secret_key = [0; 64];
    let zino_version = env!("CARGO_PKG_VERSION");
    let info = format!("ZINO:{zino_version};CHECKSUM:SHA256;HKDF:HMAC-SHA256");
    Hkdf::<Sha256>::from_prk(&app_checksum)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut secret_key)
        .expect("invalid length for Sha256 to output");
    SECRET_KEY
        .set(secret_key)
        .expect("failed to set the secret key");
}

/// Secret key.
pub(crate) static SECRET_KEY: OnceLock<[u8; 64]> = OnceLock::new();
