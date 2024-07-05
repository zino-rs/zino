//! Base64 encoding and decoding.
use base64::{engine::general_purpose::STANDARD_NO_PAD, DecodeError, Engine};

/// Encodes the data as base64 string.
#[inline]
pub(crate) fn encode(data: impl AsRef<[u8]>) -> String {
    STANDARD_NO_PAD.encode(data)
}

/// Decodes the base64-encoded data as `Vec<u8>`.
#[inline]
pub(crate) fn decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    STANDARD_NO_PAD.decode(data)
}

/// Encodes the data as base64-encoded data URL string.
#[cfg(feature = "connector-arrow")]
pub(crate) fn encode_data_url(data: impl AsRef<[u8]>) -> String {
    fn inner(bytes: &[u8]) -> String {
        let mut data = String::with_capacity(bytes.len() * 3 / 4);
        base64::engine::general_purpose::STANDARD.encode_string(bytes, &mut data);
        format!("data:text/plain;base64,{data}")
    }
    inner(data.as_ref())
}
