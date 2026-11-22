use crate::{bail, error::Error, warn};
use aes_gcm::{
    AesGcm, Nonce,
    aead::{Aead, KeyInit, consts::U12},
};
use rand::RngExt;
use sm4::Sm4;

/// Size of the `Key`.
const KEY_SIZE: usize = 16;

/// Size of the `Nonce` (Initial Vector).
const NONCE_SIZE: usize = 12;

/// AES-GCM with a 128-bit key and 96-bit nonce.
type Sm4Gcm = AesGcm<Sm4, U12>;

/// Encrypts the plaintext using `SM4`.
pub fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let cipher = Sm4Gcm::new_from_slice(&padded_key(key))?;

    let mut rng = rand::rng();
    let mut bytes = [0u8; NONCE_SIZE];
    rng.fill(&mut bytes);

    let nonce = Nonce::try_from(bytes.as_slice())?;
    let mut ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|_| warn!("fail to encrypt the plaintext"))?;
    ciphertext.extend_from_slice(&bytes);
    Ok(ciphertext)
}

/// Decrypts the data as bytes using `SM4`.
pub fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() <= NONCE_SIZE {
        bail!("invalid data length");
    }

    let cipher = Sm4Gcm::new_from_slice(&padded_key(key))?;
    let (ciphertext, bytes) = data.split_at(data.len() - NONCE_SIZE);
    let nonce = Nonce::try_from(bytes)?;
    cipher
        .decrypt(&nonce, ciphertext)
        .map_err(|_| warn!("fail to decrypt the ciphertext"))
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
