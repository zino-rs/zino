use crate::error::Error;
use cfb_mode::{Decryptor, Encryptor};
use rand::Rng;
use sm4::{
    cipher::{generic_array::GenericArray, AsyncStreamCipher, KeyIvInit},
    Sm4,
};

/// Size of the `Key`.
const KEY_SIZE: usize = 16;

/// Size of the `Nonce` (Initial Vector).
const NONCE_SIZE: usize = 16;

/// Encrypts the plaintext using `SM4`.
pub(crate) fn encrypt(plaintext: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    let mut rng = rand::thread_rng();
    let mut nonce = [0u8; NONCE_SIZE];
    rng.fill(&mut nonce);

    let mut buf = plaintext.to_vec();
    let key = padded_key(key).into();
    Encryptor::<Sm4>::new(&key, &nonce.into()).encrypt(&mut buf);
    buf.extend_from_slice(&nonce);
    Ok(buf)
}

/// Decrypts the data as bytes using `SM4`.
pub(crate) fn decrypt(data: &[u8], key: &[u8]) -> Result<Vec<u8>, Error> {
    if data.len() <= NONCE_SIZE {
        return Err(Error::new("invalid data length"));
    }

    let (ciphertext, bytes) = data.split_at(data.len() - NONCE_SIZE);
    let nonce = GenericArray::from_slice(bytes);

    let mut buf = ciphertext.to_vec();
    let key = padded_key(key).into();
    Decryptor::<Sm4>::new(&key, &nonce).decrypt(&mut buf);
    Ok(buf)
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
