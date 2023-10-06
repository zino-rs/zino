use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

/// Key derivation with HKFD-HMAC-SHA256
pub(crate) fn derive_key(info: &str, prk: &[u8]) -> [u8; 64] {
    let info = format!("{info};CHECKSUM:SHA256;HKDF:HMAC-SHA256");
    let mut okm = [0; 64];
    Hkdf::<Sha256>::from_prk(prk)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut okm)
        .expect("invalid length to output");
    okm
}

/// SHA256 digest
pub(crate) fn digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Signs the data with HMAC-SHA256
pub(crate) fn sign(data: &[u8], key: &[u8]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().into()
}
