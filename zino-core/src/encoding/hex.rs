//! Hex encoding and decoding.

/// Encodes the data as hex string.
#[inline]
pub(crate) fn encode(src: impl AsRef<[u8]>) -> String {
    faster_hex::hex_string(src.as_ref())
}

/// Decodes the hex-encoded data as `Vec<u8>`.
#[inline]
pub(crate) fn decode(src: impl AsRef<[u8]>) -> Result<Vec<u8>, faster_hex::Error> {
    let bytes = src.as_ref();
    let mut dst = vec![0; bytes.len() / 2];
    faster_hex::hex_decode(bytes, &mut dst)?;
    Ok(dst)
}
