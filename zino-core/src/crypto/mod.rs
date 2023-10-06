//! Crypto helpers for hashing, signing, encryption and decryption.

#[cfg(feature = "orm")]
mod password;

#[cfg(feature = "orm")]
pub(crate) use password::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "crypto-sm")] {
        mod sm3;
        mod sm4;

        pub(crate) use sm3::{derive_key, digest, sign};
        pub(crate) use sm4::{decrypt, encrypt};

        /// Digest type.
        pub(crate) type Hash = ::sm3::Sm3;
    } else {
        mod aes256;
        mod sha256;

        pub(crate) use aes256::{decrypt, encrypt};
        pub(crate) use sha256::{derive_key, digest, sign};

        /// Digest type.
        pub(crate) type Hash = ::sha2::Sha256;
    }
}
