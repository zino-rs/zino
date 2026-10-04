//! Base64 encoding and decoding.
use base64::{DecodeError, Engine, engine::general_purpose::STANDARD_NO_PAD};

/// Encodes the data as base64 string.
#[inline]
pub fn encode(data: impl AsRef<[u8]>) -> String {
    STANDARD_NO_PAD.encode(data)
}

/// Decodes the base64-encoded data as `Vec<u8>`.
#[inline]
pub fn decode(data: impl AsRef<[u8]>) -> Result<Vec<u8>, DecodeError> {
    STANDARD_NO_PAD.decode(data)
}

/// Encodes the data as base64-encoded data URL string.
pub fn encode_data_url(data: impl AsRef<[u8]>) -> String {
    fn inner(bytes: &[u8]) -> String {
        const PREFIX: &str = "data:text/plain;base64,";

        let b64_len = (bytes.len() + 2) * 4 / 3;
        let mut output = String::with_capacity(PREFIX.len() + b64_len);
        output.push_str(PREFIX);
        base64::engine::general_purpose::STANDARD.encode_string(bytes, &mut output);
        output
    }
    inner(data.as_ref())
}
