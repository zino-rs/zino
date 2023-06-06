//! Crypto helpers for hashing, signing, encryption and decryption.

use crate::error::Error;
use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead},
    Aes256GcmSiv, KeyInit, Nonce,
};
use rand::Rng;

#[cfg(feature = "orm")]
mod password;

#[cfg(feature = "orm")]
pub(crate) use password::*;

/// Encrypts the plaintext using `AES-GCM-SIV`.
pub(crate) fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;

    let key_padding = [key, &[0u8; KEY_SIZE]].concat();
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key_padding[0..KEY_SIZE]));

    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; NONCE_SIZE];
    rng.fill(&mut bytes);

    let nonce = Nonce::from_slice(&bytes);
    let mut ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| Error::new("fail to encrypt the plaintext"))?;
    ciphertext.extend_from_slice(&bytes);
    Ok(ciphertext)
}

/// Decrypts the data using `AES-GCM-SIV`.
pub(crate) fn decrypt(key: &[u8], data: &[u8]) -> Result<String, Error> {
    const KEY_SIZE: usize = 32;
    const NONCE_SIZE: usize = 12;

    if data.len() <= NONCE_SIZE {
        return Err(Error::new("invalid data length"));
    }

    let key_padding = [key, &[0u8; KEY_SIZE]].concat();
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&key_padding[0..KEY_SIZE]));

    let (ciphertext, bytes) = data.split_at(data.len() - NONCE_SIZE);
    let nonce = GenericArray::from_slice(bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| Error::new("fail to decrypt the ciphertext"))?;
    Ok(String::from_utf8_lossy(&plaintext).into_owned())
}
