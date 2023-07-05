use crate::{encoding::base64, error::Error};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use hmac::digest::{Digest, FixedOutput, HashMarker, Update};

/// Encrypts the hashed password using `Argon2id` and `AES-GCM-SIV`.
pub(crate) fn encrypt_hashed_password(key: &[u8], hashed_password: &[u8]) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(hashed_password, &salt)?
        .to_string();
    let ciphertext = super::encrypt(key, password_hash.as_bytes())?;
    Ok(base64::encode(ciphertext))
}

/// Encrypts the raw password using `Argon2id` and `AES-GCM-SIV`.
pub(crate) fn encrypt_raw_password<D>(key: &[u8], raw_password: &[u8]) -> Result<String, Error>
where
    D: Default + FixedOutput + HashMarker + Update,
{
    let mut hasher = D::new();
    hasher.update(raw_password);

    let hashed_password = base64::encode(hasher.finalize().as_slice());
    encrypt_hashed_password(key, hashed_password.as_bytes())
}

/// Verifies the hashed password using `Argon2id` and `AES-GCM-SIV`.
pub(crate) fn verify_hashed_password(
    key: &[u8],
    hashed_password: &[u8],
    encrypted_password: &[u8],
) -> Result<bool, Error> {
    let ciphertext = base64::decode(encrypted_password)?;
    let password_hash = super::decrypt(key, &ciphertext)?;
    let parsed_hash = PasswordHash::new(&password_hash)?;
    Argon2::default().verify_password(hashed_password, &parsed_hash)?;
    Ok(true)
}

/// Verifies the raw password using `Argon2id` and `AES-GCM-SIV`.
pub(crate) fn verify_raw_password<D>(
    key: &[u8],
    raw_password: &[u8],
    encrypted_password: &[u8],
) -> Result<bool, Error>
where
    D: Default + FixedOutput + HashMarker + Update,
{
    let mut hasher = D::new();
    hasher.update(raw_password);

    let hashed_password = base64::encode(hasher.finalize().as_slice());
    verify_hashed_password(key, hashed_password.as_bytes(), encrypted_password)
}
