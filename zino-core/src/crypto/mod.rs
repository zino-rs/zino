//! Crypto helpers for hashing, signing, encryption and decryption.

#[cfg(feature = "orm")]
mod password;

#[cfg(feature = "orm")]
pub(crate) use password::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "crypto-sm")] {
        mod sm3;
        mod sm4;

        pub(crate) use sm3::{digest, derive_key};
        pub(crate) use sm4::{encrypt, decrypt};
    } else {
        mod aes256;
        mod sha256;

        pub(crate) use aes256::{encrypt, decrypt};
        pub(crate) use sha256::{digest, derive_key};
    }
}
