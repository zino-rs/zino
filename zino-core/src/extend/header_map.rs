use http::header::{HeaderMap, ToStrError};

/// Extension trait for [`HeaderMap`](http::HeaderMap).
pub trait HeaderMapExt {
    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Parses the essence of the `content-type` header, discarding the optional parameters.
    fn parse_content_type(&self) -> Result<Option<&str>, ToStrError>;
}

impl HeaderMapExt for HeaderMap {
    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }

    fn parse_content_type(&self) -> Result<Option<&str>, ToStrError> {
        match self.get("content-type") {
            Some(header_value) => {
                let mut content_type = header_value.to_str()?;
                if let Some((essence, _)) = content_type.split_once(';') {
                    content_type = essence;
                }
                Ok(Some(content_type))
            }
            None => Ok(None),
        }
    }
}
