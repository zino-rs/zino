use sha1::{Digest, Sha1};

/// SHA1 digest
pub(crate) fn checksum(data: &[u8]) -> [u8; 20] {
    let mut hasher = Sha1::new();
    hasher.update(data);
    hasher.finalize().into()
}
