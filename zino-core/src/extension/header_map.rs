use crate::format;
use http::header::HeaderMap;
use std::net::IpAddr;

/// Extension trait for [`HeaderMap`](http::HeaderMap).
pub trait HeaderMapExt {
    /// Ges the string corresponding to the key.
    fn get_str(&self, key: &str) -> Option<&str>;

    /// Gets the destination host.
    ///
    /// It is determined in the following priority:
    ///
    /// 1. `Forwarded` header `host` key
    /// 2. The first `X-Forwarded-Host` header
    /// 3. `Host` header
    fn get_host(&self) -> Option<&str> {
        self.get_str("forwarded")
            .and_then(|s| {
                s.split(';').find_map(|entry| {
                    let parts = entry.split('=').collect::<Vec<_>>();
                    (parts.len() == 2 && parts[0].eq_ignore_ascii_case("host")).then(|| parts[1])
                })
            })
            .or_else(|| {
                self.get_str("x-forwarded-host")
                    .and_then(|s| s.split(',').next())
            })
            .or_else(|| self.get_str("host"))
    }

    /// Gets the client's remote IP.
    ///
    /// It is determined in the following priority:
    ///
    /// 1. `Forwarded` header `for` key
    /// 2. The first `X-Forwarded-For` header
    fn get_client_ip(&self) -> Option<IpAddr> {
        self.get_str("forwarded")
            .and_then(|s| {
                s.split(';').find_map(|entry| {
                    let parts = entry.split('=').collect::<Vec<_>>();
                    (parts.len() == 2 && parts[0].eq_ignore_ascii_case("for")).then(|| parts[1])
                })
            })
            .or_else(|| {
                self.get_str("x-forwarded-for")
                    .and_then(|s| s.split(',').next())
            })
            .and_then(|s| s.parse().ok())
    }

    /// Gets the essence of the `content-type` header, discarding the optional parameters.
    fn get_content_type(&self) -> Option<&str> {
        self.get_str("content-type").map(|content_type| {
            if let Some((essence, _)) = content_type.split_once(';') {
                essence
            } else {
                content_type
            }
        })
    }

    /// Checks whether it has a `content-type: application/json` or similar header.
    fn has_json_content_type(&self) -> bool {
        if let Some(content_type) = self.get_str("content-type") {
            format::header::check_json_content_type(content_type)
        } else {
            false
        }
    }
}

impl HeaderMapExt for HeaderMap {
    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }
}
