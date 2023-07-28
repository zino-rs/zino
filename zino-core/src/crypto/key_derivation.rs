use hkdf::Hkdf;
use sha2::Sha256;

pub(crate) fn hkdf_sha256(info: &[u8], prk: &[u8]) -> [u8; 64] {
    let mut okm = [0; 64];
    Hkdf::<Sha256>::from_prk(prk)
        .expect("pseudorandom key is not long enough")
        .expand(info, &mut okm)
        .expect("invalid length for Sha256 to output");
    okm
}
