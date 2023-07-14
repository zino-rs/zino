use super::Application;
use crate::extension::TomlTableExt;
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use std::{env, sync::OnceLock};

/// Initializes the secret key.
pub(super) fn init<APP: Application + ?Sized>() {
    let checksum: [u8; 32] = APP::config()
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().try_into().ok())
        .unwrap_or_else(|| {
            let pkg_name = env::var("CARGO_PKG_NAME").expect("fail to get crate name");
            let pkg_version = env::var("CARGO_PKG_VERSION").expect("fail to get crate version");
            let pkg_key = format!("{pkg_name}@{pkg_version}");
            let mut hasher = Sha256::new();
            hasher.update(pkg_key.as_bytes());
            hasher.finalize().into()
        });

    let mut secret_key = [0; 64];
    let info = "ZINO:APPLICATION;CHECKSUM:SHA256;HKDF:HMAC-SHA256";
    Hkdf::<Sha256>::from_prk(&checksum)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut secret_key)
        .expect("invalid length for Sha256 to output");
    SECRET_KEY
        .set(secret_key)
        .expect("fail to set the secret key");
}

/// Secret key.
pub(crate) static SECRET_KEY: OnceLock<[u8; 64]> = OnceLock::new();
