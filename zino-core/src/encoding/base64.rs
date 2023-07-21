//! Base64 encoding and decoding.
use base64_simd::{Error, STANDARD_NO_PAD};

/// Encodes the data as base64 string.
#[inline]
pub(crate) fn encode(data: impl AsRef<[u8]>) -> String {
    STANDARD_NO_PAD.encode_to_string(data)
}

/// Decodes the base64-encoded data as `Vec<u8>`.
#[inline]
pub(crate) fn decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, Error> {
    base64_simd::forgiving_decode_to_vec(data.as_ref())
}

/// Encodes the data as base64-encoded data URL string.
#[cfg(feature = "connector-arrow")]
pub(crate) fn encode_data_url(data: impl AsRef<[u8]>) -> String {
    let bytes = data.as_ref();
    let mut data = String::with_capacity(bytes.len() * 3 / 4);
    base64_simd::STANDARD.encode_append(bytes, &mut data);
    format!("data:text/plain;base64,{data}")
}
