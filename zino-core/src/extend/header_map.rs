use crate::SharedString;
use http::header::{self, HeaderMap};
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
    fn get_host(&self) -> Option<&str>;

    /// Gets the client's remote IP.
    ///
    /// It is determined in the following priority:
    ///
    /// 1. `Forwarded` header `for` key
    /// 2. The first `X-Forwarded-For` header
    fn get_client_ip(&self) -> Option<IpAddr>;

    /// Gets the essence of the `content-type` header, discarding the optional parameters.
    fn get_content_type(&self) -> Option<&str>;

    /// Gets the data type by parsing the `content-type` header.
    fn get_data_type(&self) -> Option<SharedString>;

    /// Checks whether it has a `content-type: application/json` or similar header.
    fn has_json_content_type(&self) -> bool;

    /// Selects a language from the supported locales by parsing and comparing
    /// the `accept-language` header.
    fn select_language<'a>(&'a self, supported_locales: &[&'a str]) -> Option<&'a str>;
}

impl HeaderMapExt for HeaderMap {
    #[inline]
    fn get_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(|v| v.to_str().ok())
    }

    fn get_host(&self) -> Option<&str> {
        self.get(header::FORWARDED)
            .and_then(|v| v.to_str().ok())
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
            .or_else(|| self.get(header::HOST).and_then(|v| v.to_str().ok()))
    }

    fn get_client_ip(&self) -> Option<IpAddr> {
        self.get(header::FORWARDED)
            .and_then(|v| v.to_str().ok())
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
        let content_type = self.get_content_type()?;
        let data_type = match content_type {
            "application/json" | "application/problem+json" => "json".into(),
            "application/jsonlines" | "application/x-ndjson" => "ndjson".into(),
            "application/msgpack" | "application/x-msgpack" => "msgpack".into(),
            "application/octet-stream" => "bytes".into(),
            "application/x-www-form-urlencoded" => "form".into(),
            "multipart/form-data" => "multipart".into(),
            "text/csv" => "csv".into(),
            "text/plain" => "text".into(),
            _ => {
                if content_type.starts_with("application/") && content_type.ends_with("+json") {
                    "json".into()
                } else {
                    content_type.to_owned().into()
                }
            }
        };
        Some(data_type)
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

    fn select_language<'a>(&'a self, supported_locales: &[&'a str]) -> Option<&'a str> {
        let mut languages = self
            .get(header::ACCEPT_LANGUAGE)
            .and_then(|v| v.to_str().ok())?
            .split(',')
            .filter_map(|s| {
                let (language, quality) = if let Some((language, quality)) = s.split_once(';') {
                    let quality = quality.trim().strip_prefix("q=")?.parse::<f32>().ok()?;
                    (language.trim(), quality)
                } else {
                    (s.trim(), 1.0)
                };
                supported_locales.iter().find_map(|&locale| {
                    (locale.eq_ignore_ascii_case(language) || locale.starts_with(language))
                        .then_some((locale, quality))
                })
            })
            .collect::<Vec<_>>();
        languages.sort_by(|a, b| b.1.total_cmp(&a.1));
        languages.first().map(|&(language, _)| language)
    }
}

#[cfg(test)]
mod tests {
    use super::HeaderMapExt;
    use http::header::{self, HeaderMap, HeaderValue};

    #[test]
    fn it_selects_language() {
        let mut headers = HeaderMap::new();
        let header_value = "zh-CN,zh;q=0.9,en;q=0.8,en-US;q=0.7";
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static(header_value),
        );
        assert_eq!(headers.select_language(&["en-US", "zh-CN"]), Some("zh-CN"),);

        let header_value = "zh-HK,zh;q=0.8,en-US; q=0.7";
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static(header_value),
        );
        assert_eq!(headers.select_language(&["en-US", "zh-CN"]), Some("zh-CN"),);

        let header_value = "zh-HK, zh;q=0.8,en-US; q=0.9";
        headers.insert(
            header::ACCEPT_LANGUAGE,
            HeaderValue::from_static(header_value),
        );
        assert_eq!(headers.select_language(&["en-US", "zh-CN"]), Some("en-US"),);
    }
}
