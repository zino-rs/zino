use super::Schema;
use crate::{
    crypto, encoding::base64, error::Error, extension::TomlTableExt, openapi, state::State, Map,
};
use sha2::Sha256;
use std::{fmt::Display, sync::LazyLock};

/// Helper utilities for models.
pub trait ModelHelper<K>: Schema<PrimaryKey = K>
where
    K: Default + Display + PartialEq,
{
    /// Returns the secret key for the model.
    /// It should have at least 64 bytes.
    ///
    /// # Note
    ///
    /// This should only be used for internal services. Do not expose it to external users.
    #[inline]
    fn secret_key() -> &'static [u8] {
        SECRET_KEY.as_slice()
    }

    /// Encrypts the password for the model.
    fn encrypt_password(passowrd: &str) -> Result<String, Error> {
        let key = Self::secret_key();
        let passowrd = passowrd.as_bytes();
        if let Ok(bytes) = base64::decode(passowrd) && bytes.len() == 256 {
            crypto::encrypt_hashed_password(passowrd, key)
        } else {
            crypto::encrypt_raw_password::<Sha256>(passowrd, key)
        }
    }

    /// Verifies the password for the model.
    fn verify_password(passowrd: &str, encrypted_password: &str) -> Result<bool, Error> {
        let key = Self::secret_key();
        let passowrd = passowrd.as_bytes();
        let encrypted_password = encrypted_password.as_bytes();
        if let Ok(bytes) = base64::decode(passowrd) && bytes.len() == 256 {
            crypto::verify_hashed_password(passowrd, encrypted_password, key)
        } else {
            crypto::verify_raw_password::<Sha256>(passowrd, encrypted_password, key)
        }
    }

    /// Translates the model data.
    #[inline]
    fn translate_model(model: &mut Map) {
        openapi::translate_model_entry(model, Self::model_name());
    }
}

impl<M, K> ModelHelper<K> for M
where
    M: Schema<PrimaryKey = K>,
    K: Default + Display + PartialEq,
{
}

/// Secret key.
static SECRET_KEY: LazyLock<[u8; 64]> = LazyLock::new(|| {
    let config = State::shared()
        .get_config("database")
        .expect("the `database` field should be a table");
    let checksum: [u8; 32] = config
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().try_into().ok())
        .unwrap_or_else(|| {
            let driver_name = format!("{}_{}", *super::NAMESPACE_PREFIX, super::DRIVER_NAME);
            crypto::sha256(driver_name.as_bytes())
        });
    crypto::hkdf_sha256(b"ZINO:ORM;CHECKSUM:SHA256;HKDF:HMAC-SHA256", &checksum)
});
