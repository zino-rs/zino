//! Base64 encoding and decoding.
use base64::{engine::general_purpose::STANDARD_NO_PAD, DecodeError, Engine};

/// Encodes the data as base64 string.
#[inline]
pub(crate) fn encode(data: impl AsRef<[u8]>) -> String {
    STANDARD_NO_PAD.encode(data)
}

/// Decodes the base64 encoded data as `Vec<u8>`.
#[inline]
pub(crate) fn decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    STANDARD_NO_PAD.decode(data)
}
