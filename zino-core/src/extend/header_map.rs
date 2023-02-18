use crate::SharedString;
use http::header::{self, HeaderMap};

/// Extension trait for [`HeaderMap`](http::HeaderMap).
pub trait HeaderMapExt {
    /// Extracts the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Extracts the essence of the `content-type` header, discarding the optional parameters.
    fn get_content_type(&self) -> Option<&str>;

    /// Gets the data type by parsing the `content-type` header.
    fn get_data_type(&self) -> Option<SharedString>;

    /// Checks whether it has a `content-type: application/json` or similar header.
    fn has_json_content_type(&self) -> bool;
}

impl HeaderMapExt for HeaderMap {
    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }

    fn get_content_type(&self) -> Option<&str> {
        self.get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|content_type| {
                if let Some((essence, _)) = content_type.split_once(';') {
                    essence
                } else {
                    content_type
                }
            })
    }

    fn get_data_type(&self) -> Option<SharedString> {
        self.get_content_type()
            .map(|content_type| match content_type {
                "application/json" => "json".into(),
                "application/octet-stream" => "bytes".into(),
                "application/x-www-form-urlencoded" => "form".into(),
                "multipart/form-data" => "multipart".into(),
                "text/plain" => "text".into(),
                _ => {
                    if content_type.starts_with("application/") && content_type.ends_with("+json") {
                        "json".into()
                    } else {
                        content_type.to_owned().into()
                    }
                }
            })
    }

    fn has_json_content_type(&self) -> bool {
        if let Some(content_type) = self.get(header::CONTENT_TYPE).and_then(|v| v.to_str().ok()) {
            let essence = if let Some((essence, _)) = content_type.split_once(';') {
                essence
            } else {
                content_type
            };
            essence == "application/json"
                || (essence.starts_with("application/") && essence.ends_with("+json"))
        } else {
            false
        }
    }
}
