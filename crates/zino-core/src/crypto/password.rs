use crate::{encoding::base64, error::Error};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Encrypts the hashed password using `Argon2id`.
pub(crate) fn encrypt_hashed_password(hashed_password: &[u8], key: &[u8]) -> Result<String, Error> {
    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(hashed_password, &salt)?
        .to_string();
    let ciphertext = super::encrypt(password_hash.as_bytes(), key)?;
    Ok(base64::encode(ciphertext))
}

/// Encrypts the raw password using `Argon2id`.
pub(crate) fn encrypt_raw_password(raw_password: &[u8], key: &[u8]) -> Result<String, Error> {
    let hashed_password = base64::encode(super::digest(raw_password));
    encrypt_hashed_password(hashed_password.as_bytes(), key)
}

/// Verifies the hashed password using `Argon2id`.
pub(crate) fn verify_hashed_password(
    hashed_password: &[u8],
    encrypted_password: &[u8],
    key: &[u8],
) -> Result<bool, Error> {
    let ciphertext = base64::decode(encrypted_password)?;
    let password_hash = super::decrypt(&ciphertext, key)?;
    let password_hash_str = String::from_utf8_lossy(&password_hash);
    let parsed_hash = PasswordHash::new(&password_hash_str)?;
    Argon2::default().verify_password(hashed_password, &parsed_hash)?;
    Ok(true)
}

/// Verifies the raw password using `Argon2id`.
pub(crate) fn verify_raw_password(
    raw_password: &[u8],
    encrypted_password: &[u8],
    key: &[u8],
) -> Result<bool, Error> {
    let hashed_password = base64::encode(super::digest(raw_password));
    verify_hashed_password(hashed_password.as_bytes(), encrypted_password, key)
}
