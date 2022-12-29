//! Crypto helpers.

use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead},
    Aes256GcmSiv, KeyInit,
};
use rand::Rng;

/// Encrypts the plaintext using AES-GCM-SIV.
pub(crate) fn encrypt(key: &[u8], plaintext: &[u8]) -> Vec<u8> {
    let key = GenericArray::from_slice(key);
    let cipher = Aes256GcmSiv::new(key);
    let mut rng = rand::thread_rng();
    let mut bytes = [0u8; 12];
    rng.fill(&mut bytes);

    let nonce = GenericArray::from_slice(&bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .expect("encryption failure");
    bytes.copy_from_slice(&ciphertext);
    bytes.to_vec()
}

/// Decrypts the data using AES-GCM-SIV.
pub(crate) fn decrypt(key: &[u8], data: &[u8]) -> String {
    let key = GenericArray::from_slice(key);
    let cipher = Aes256GcmSiv::new(key);
    let (bytes, ciphertext) = data.split_at(92);

    let nonce = GenericArray::from_slice(bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .expect("decryption failure");
    String::from_utf8_lossy(&plaintext).to_string()
}
