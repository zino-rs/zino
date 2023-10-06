use crate::error::Error;
use libsm::sm4::{Cipher, Mode};
use rand::Rng;

/// Size of the `Key`.
const KEY_SIZE: usize = 16;

/// Size of the `Nonce` (Initial Vector).
const NONCE_SIZE: usize = 16;

/// Encrypts the plaintext using `SM4`.
pub(crate) fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let key_padding = [key, &[0u8; KEY_SIZE]].concat();
    let cipher = Cipher::new(&key_padding[0..KEY_SIZE], Mode::Cfb)?;

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

    let key_padding = [key, &[0u8; KEY_SIZE]].concat();
    let cipher = Cipher::new(&key_padding[0..KEY_SIZE], Mode::Cfb)?;

    let (ciphertext, nonce) = data.split_at(data.len() - NONCE_SIZE);
    cipher.decrypt(ciphertext, nonce).map_err(|err| {
        let message = format!("fail to decrypt the ciphertext: {err}");
        Error::new(message)
    })
}
