use crate::error::Error;
use libsm::sm4::{Cipher, Mode};
use rand::Rng;

/// Size of the `Key`.
const KEY_SIZE: usize = 16;

/// Size of the `Nonce` (Initial Vector).
const NONCE_SIZE: usize = 16;

/// Encrypts the plaintext using `SM4`.
pub(crate) fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Cipher::new(&padded_key(key), Mode::Cfb)?;
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce);

    let mut ciphertext = cipher.encrypt(plaintext, &nonce)?;
    ciphertext.extend_from_slice(&nonce);
    Ok(ciphertext)
}

/// Decrypts the data as bytes using `SM4`.
pub(crate) fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() <= NONCE_SIZE {
        return Err(Error::new("invalid data length"));
    }

    let cipher = Cipher::new(&padded_key(key), Mode::Cfb)?;
    let (ciphertext, nonce) = data.split_at(data.len() - NONCE_SIZE);
    cipher.decrypt(ciphertext, nonce).map_err(|err| {
        let message = format!("fail to decrypt the ciphertext: {err}");
        Error::new(message)
    })
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
