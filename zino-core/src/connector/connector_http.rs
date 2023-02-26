use super::{Connector, DataSource, DataSourceConnector::Http};
use crate::{
    application::http_client,
    extend::{AvroRecordExt, HeaderMapExt, JsonObjectExt, TomlTableExt},
    format,
    trace::TraceContext,
    BoxError, Map, Record,
};
use http::{
    header::{HeaderMap, HeaderName},
    Method,
};
use reqwest::Response;
use serde::de::DeserializeOwned;
use serde_json::{value::RawValue, Value};
use toml::Table;
use url::Url;

/// A connector to HTTP services.
pub struct HttpConnector {
    /// HTTP request method (VERB).
    method: Method,
    /// Base URL.
    base_url: Url,
    /// HTTP request headers.
    headers: Map,
    /// Optional request body.
    body: Option<Box<RawValue>>,
}

impl HttpConnector {
    /// Constructs a new instance, returning an error if it fails.
    pub fn try_new(method: &str, base_url: &str) -> Result<Self, BoxError> {
        Ok(Self {
            method: method.parse()?,
            base_url: base_url.parse()?,
            headers: Map::new(),
            body: None,
        })
    }

    /// Returns the request method.
    #[inline]
    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    /// Returns the optional body.
    #[inline]
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref().map(|raw_value| raw_value.get())
    }

    /// Makes an HTTP request with the given query and params.
    pub async fn fetch(&self, query: &str, params: Option<&Map>) -> Result<Response, BoxError> {
        let url = self.base_url.join(query)?;
        let resource = format::format_query(url.as_str(), params);
        let mut options = Map::new();
        options.upsert("method", self.method());
        if let Some(body) = self.body() {
            options.upsert("body", format::format_query(body, params));
        }

        let mut headers = HeaderMap::new();
        for (key, value) in self.headers.iter() {
            if let Ok(header_name) = HeaderName::try_from(key) {
                if let Some(header_value) = value
                    .as_str()
                    .and_then(|s| format::format_query(s, params).parse().ok())
                {
                    headers.insert(header_name, header_value);
                }
            }
        }

        let mut trace_context = TraceContext::new();
        let span_id = trace_context.span_id();
        trace_context
            .trace_state_mut()
            .push("zino", format!("{span_id:x}"));
        http_client::request_builder(resource.as_ref(), Some(&options))?
            .headers(headers)
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await
            .map_err(BoxError::from)
    }

    /// Makes an HTTP request with the given query and params,
    /// and deserializes the response body as JSON.
    pub async fn fetch_json<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<T, BoxError> {
        let response = self.fetch(query, params).await?.error_for_status()?;
        let data = if response.headers().has_json_content_type() {
            response.json().await?
        } else {
            let text = response.text().await?;
            serde_json::from_str(&text)?
        };
        Ok(data)
    }
}

impl Connector for HttpConnector {
    fn try_new_data_source(config: &Table) -> Result<DataSource, BoxError> {
        let name = config.get_str("name").unwrap_or("http");
        let catalog = config.get_str("catalog").unwrap_or(name);

        let method = config.get_str("method").unwrap_or_default();
        let base_url = config.get_str("base-url").unwrap_or_default();
        let connector = HttpConnector::try_new(method, base_url)?;
        let data_source = DataSource::new("http", None, name, catalog, Http(connector));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, BoxError> {
        if let Value::Object(map) = self.fetch_json(query, params).await? &&
            let Some(rows_affected) = map
                .get_u64("rows_affected")
                .or_else(|| map.get_u64("total_rows"))
        {
            Ok(Some(rows_affected))
        } else {
            Ok(None)
        }
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, BoxError> {
        let records = match self.fetch_json(query, params).await? {
            Value::Array(vec) => vec
                .into_iter()
                .filter_map(|value| {
                    if let Value::Object(map) = value {
                        Some(map.into_record())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>(),
            Value::Object(mut map) => {
                if let Some(value) = map.remove("data").or_else(|| map.remove("result")) {
                    if let Value::Array(vec) = value {
                        vec.into_iter()
                            .filter_map(|value| {
                                if let Value::Object(map) = value {
                                    Some(map.into_record())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    } else {
                        let mut record = Record::new();
                        record.upsert("data", value);
                        vec![record]
                    }
                } else {
                    vec![map.into_record()]
                }
            }
            _ => return Err("invalid data format".into()),
        };
        Ok(records)
    }

    async fn query_one(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<Record>, BoxError> {
        let record = match self.fetch_json(query, params).await? {
            Value::Object(mut map) => {
                if let Some(value) = map.remove("data").or_else(|| map.remove("result")) {
                    if let Value::Object(data) = value {
                        data.into_record()
                    } else {
                        let mut record = Record::new();
                        record.upsert("data", value);
                        record
                    }
                } else {
                    map.into_record()
                }
            }
            _ => return Err("invalid data format".into()),
        };
        Ok(Some(record))
    }
}
