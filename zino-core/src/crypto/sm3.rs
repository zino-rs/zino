use libsm::sm3::hash::Sm3Hash;

/// Key derivation with HKFD-HMAC-SM3
///
/// Reference: https://docs.rs/hkdf/latest/hkdf/struct.Hkdf.html#method.expand_multi_info
pub(crate) fn derive_key(info: &str, prk: &[u8]) -> [u8; 64] {
    const OUTPUT_SIZE: usize = 32;

    let info = format!("{info};CHECKSUM:SM3;HKDF:HMAC-SM3");
    let bytes = info.as_bytes();
    let data_len = OUTPUT_SIZE + bytes.len() + 1;

    let mut okm = [0; 64];
    let mut prev: Option<[u8; OUTPUT_SIZE]> = None;
    for (block_n, block) in okm.chunks_mut(OUTPUT_SIZE).enumerate() {
        let mut data = Vec::with_capacity(data_len);
        if let Some(prev) = prev {
            data.extend_from_slice(&prev);
        };
        data.extend_from_slice(bytes);
        data.push(block_n as u8 + 1);

        let output = sign(&data, prk);
        let block_len = block.len();
        block.copy_from_slice(&output[..block_len]);
        prev = Some(output);
    }
    okm
}

/// SM3 digest
#[inline]
pub(crate) fn digest(data: &[u8]) -> [u8; 32] {
    Sm3Hash::new(data).get_hash()
}

/// Signs the data with HMAC-SM3
///
/// Reference: https://docs.rs/hmac-sm3/latest/hmac_sm3/struct.HmacSm3.html
pub(crate) fn sign(data: &[u8], key: &[u8]) -> [u8; 32] {
    const BLOCK_SIZE: usize = 64;
    const OUTPUT_SIZE: usize = 32;

    let mut structured_key = [0_u8; BLOCK_SIZE];
    let key_len = key.len();
    if key_len > BLOCK_SIZE {
        structured_key[0..OUTPUT_SIZE].copy_from_slice(&digest(key));
    } else {
        structured_key[0..key_len].copy_from_slice(key);
    }

    let mut ipad = [0x36_u8; BLOCK_SIZE];
    let mut opad = [0x5c_u8; BLOCK_SIZE];
    for i in 0..BLOCK_SIZE {
        ipad[i] ^= structured_key[i];
        opad[i] ^= structured_key[i];
    }

    let mut ipad_message = Vec::with_capacity(data.len() + BLOCK_SIZE);
    ipad_message.extend_from_slice(&ipad);
    ipad_message.extend_from_slice(data);

    let ipad_message_digest = digest(&ipad_message);
    let mut message = [0_u8; BLOCK_SIZE + OUTPUT_SIZE];
    message[0..BLOCK_SIZE].copy_from_slice(&opad);
    message[BLOCK_SIZE..].copy_from_slice(&ipad_message_digest);
    digest(&message)
}

#[cfg(test)]
mod tests {
    use crate::encoding::hex;
    use hmac::{Hmac, Mac};

    #[test]
    fn it_signs_with_hmac_sm3() {
        let data = b"Hello World";
        let key = b"TestSecret";
        assert_eq!(
            hex::encode(super::sign(data, key)),
            "9d91da552268ddf11b9f69662773a66c6375b250336dfb9293e7e2611c36d79f",
        );

        let mut mac =
            Hmac::<crate::crypto::Hash>::new_from_slice(key).expect("HMAC can take key of any size");
        mac.update(data);
        assert_eq!(
            hex::encode(mac.finalize().into_bytes()),
            "9d91da552268ddf11b9f69662773a66c6375b250336dfb9293e7e2611c36d79f",
        );
    }
}
