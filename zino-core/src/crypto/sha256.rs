use hkdf::Hkdf;
use sha2::{Digest, Sha256};

/// Sha256 digest
pub(crate) fn digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Key derivation
pub(crate) fn derive_key(info: &str, prk: &[u8]) -> [u8; 64] {
    let info = format!("{info};CHECKSUM:SHA256;HKDF:HMAC-SHA256");
    let mut okm = [0; 64];
    Hkdf::<Sha256>::from_prk(prk)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut okm)
        .expect("invalid length to output");
    okm
}
