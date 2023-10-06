use crate::error::Error;
use aes_gcm_siv::{
    aead::{generic_array::GenericArray, Aead},
    Aes256GcmSiv, KeyInit, Nonce,
};
use rand::Rng;

/// Size of the `Key`.
const KEY_SIZE: usize = 32;

/// Size of the `Nonce` (Initial Vector).
const NONCE_SIZE: usize = 12;

/// Encrypts the plaintext using `AES-GCM-SIV`.
pub(crate) fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&padded_key(key)));

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

/// Decrypts the data as bytes using `AES-GCM-SIV`.
pub(crate) fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() <= NONCE_SIZE {
        return Err(Error::new("invalid data length"));
    }

    let cipher = Aes256GcmSiv::new(GenericArray::from_slice(&padded_key(key)));

    let (ciphertext, bytes) = data.split_at(data.len() - NONCE_SIZE);
    let nonce = GenericArray::from_slice(bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| Error::new("fail to decrypt the ciphertext"))
}

/// Gets the padded key.
fn padded_key(key: &[u8]) -> [u8; KEY_SIZE] {
    let mut padded_key = [0_u8; KEY_SIZE];
    let key_len = key.len();
    if key_len > KEY_SIZE {
        padded_key.copy_from_slice(&key[0..KEY_SIZE]);
    } else {
        padded_key[0..key_len].copy_from_slice(key);
    }
    padded_key
}
