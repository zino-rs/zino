use hkdf::Hkdf;
use sm3::{Digest, Sm3};

/// Key derivation with HKFD-HMAC-SM3
pub(crate) fn derive_key(info: &str, prk: &[u8]) -> [u8; 64] {
    let info = format!("{info};CHECKSUM:SM3;HKDF:HMAC-SM3");
    let mut okm = [0; 64];
    Hkdf::<Sm3>::from_prk(prk)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut okm)
        .expect("invalid length to output");
    okm
}

/// SM3 digest
#[inline]
pub(crate) fn digest(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sm3::new();
    hasher.update(data);
    hasher.finalize().into()
}
