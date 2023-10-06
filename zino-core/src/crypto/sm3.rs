use libsm::sm3::hash::Sm3Hash;

/// SM3 digest
#[inline]
pub(crate) fn digest(data: &[u8]) -> [u8; 32] {
    Sm3Hash::new(data).get_hash()
}

/// Key derivation
///
/// Reference: https://docs.rs/hkdf/latest/hkdf/struct.Hkdf.html#method.expand_multi_info
pub(crate) fn derive_key(info: &str, prk: &[u8]) -> [u8; 64] {
    const OUTPUT_SIZE: usize = 32;

    let info = format!("{info};CHECKSUM:SM3;HKDF:HMAC-SM3");
    let bytes = info.as_bytes();
    let data_len = prk.len() + OUTPUT_SIZE + bytes.len() + 1;

    let mut okm = [0; 64];
    let mut prev: Option<[u8; OUTPUT_SIZE]> = None;
    for (block_n, block) in okm.chunks_mut(OUTPUT_SIZE).enumerate() {
        let mut data = Vec::with_capacity(data_len);
        data.extend_from_slice(prk);
        if let Some(prev) = prev {
            data.extend_from_slice(&prev);
        };
        data.extend_from_slice(bytes);
        data.push(block_n as u8 + 1);

        let output = digest(&data);
        let block_len = block.len();
        block.copy_from_slice(&output[..block_len]);
        prev = Some(output);
    }
    okm
}
