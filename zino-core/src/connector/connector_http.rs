use super::{Connector, DataSource, DataSourceConnector::Http};
use crate::{
    application::http_client,
    bail,
    error::Error,
    extension::{
        AvroRecordExt, HeaderMapExt, JsonObjectExt, JsonValueExt, TomlTableExt, TomlValueExt,
    },
    helper,
    trace::TraceContext,
    warn, JsonValue, Map, Record,
};
use http::{
    header::{HeaderMap, HeaderName},
    Method,
};
use reqwest::Response;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::value::RawValue;
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
    /// JSON Pointer for looking up a value from the response data.
    json_pointer: Option<String>,
}

impl HttpConnector {
    /// Constructs a new instance, returning an error if it fails.
    pub fn try_new(method: &str, base_url: &str) -> Result<Self, Error> {
        Ok(Self {
            method: method.parse()?,
            base_url: base_url.parse()?,
            headers: Map::new(),
            body: None,
            json_pointer: None,
        })
    }

    /// Attempts to construct a new instance from the config.
    #[inline]
    pub fn try_with_config(config: &Table) -> Result<Self, Error> {
        let method = config.get_str("method").unwrap_or("GET");
        let base_url = config
            .get_str("base-url")
            .ok_or_else(|| warn!("the base URL should be specified"))?;

        let mut connector = HttpConnector::try_new(method, base_url)?;
        let headers = config.get("headers").map(|v| v.to_json_value());
        if let Some(JsonValue::Object(headers)) = headers {
            connector.headers = headers;
        }
        if let Some(body) = config.get_table("body") {
            let raw_value = serde_json::value::to_raw_value(body)?;
            connector.body = Some(raw_value);
        }
        if let Some(json_pointer) = config.get_str("json-pointer") {
            connector.json_pointer = Some(json_pointer.into());
        }

        Ok(connector)
    }

    /// Inserts a key/value pair into the request headers.
    #[inline]
    pub fn insert_header(&mut self, key: &str, value: impl Into<JsonValue>) {
        self.headers.upsert(key, value.into());
    }

    /// Sets the request query.
    #[inline]
    pub fn set_query<T: Serialize>(&mut self, query: &T) {
        if let Ok(query) = serde_qs::to_string(query) {
            self.base_url.set_query(Some(&query));
        }
    }

    /// Sets the request body.
    #[inline]
    pub fn set_body<T: Serialize>(&mut self, body: &T) {
        self.body = serde_json::value::to_raw_value(body).ok();
    }

    /// Sets a JSON Pointer for looking up a value from the response data.
    /// It only applies when the response data is a JSON object.
    #[inline]
    pub fn set_json_pointer(&mut self, pointer: impl Into<String>) {
        self.json_pointer = Some(pointer.into());
    }

    /// Makes an HTTP request with the given query and params.
    pub async fn fetch(&self, query: &str, params: Option<&Map>) -> Result<Response, Error> {
        let mut url = self.base_url.clone();
        url.set_query(Some(query));

        let resource = helper::format_query(url.as_str(), params);
        let mut options = Map::from_entry("method", self.method.as_str());
        if let Some(body) = self.body.as_deref().map(|v| v.get()) {
            options.upsert("body", helper::format_query(body, params));
        }

        let mut headers = HeaderMap::new();
        for (key, value) in self.headers.iter() {
            if let Ok(header_name) = HeaderName::try_from(key) {
                let header_value = value
                    .as_str()
                    .and_then(|s| helper::format_query(s, params).parse().ok());
                if let Some(header_value) = header_value {
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
            .map_err(Error::from)
    }

    /// Makes an HTTP request with the given query and params,
    /// and deserializes the response body via JSON.
    pub async fn fetch_json<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<T, Error> {
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
    fn try_new_data_source(config: &Table) -> Result<DataSource, Error> {
        let name = config.get_str("name").unwrap_or("http");
        let catalog = config.get_str("catalog").unwrap_or(name);

        let connector = HttpConnector::try_with_config(config)?;
        let data_source = DataSource::new("http", None, name, catalog, Http(connector));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error> {
        let data: JsonValue = self.fetch_json(query, params).await?;
        let rows_affected = data.into_map_opt().and_then(|map| {
            map.get_u64("total")
                .or_else(|| map.get_u64("total_rows"))
                .or_else(|| map.get_u64("rows_affected"))
                .or_else(|| map.get_u64("num_entries"))
                .or_else(|| map.get_u64("num_items"))
        });
        Ok(rows_affected)
    }

    async fn query(&self, query: &str, params: Option<&Map>) -> Result<Vec<Record>, Error> {
        let records = match self.fetch_json(query, params).await? {
            JsonValue::Array(vec) => vec
                .into_iter()
                .filter_map(|v| v.into_map_opt())
                .map(|m| m.into_avro_record())
                .collect::<Vec<_>>(),
            JsonValue::Object(mut map) => {
                let data = if let Some(json_pointer) = &self.json_pointer {
                    map.pointer(json_pointer).cloned()
                } else {
                    map.remove("data").or_else(|| map.remove("result"))
                };
                if let Some(value) = data {
                    if let JsonValue::Array(vec) = value {
                        vec.into_iter()
                            .filter_map(|v| v.into_map_opt())
                            .map(|m| m.into_avro_record())
                            .collect::<Vec<_>>()
                    } else {
                        vec![Record::from_entry("data", value)]
                    }
                } else {
                    vec![map.into_avro_record()]
                }
            }
            _ => bail!("invalid data format"),
        };
        Ok(records)
    }

    async fn query_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let data = match self.fetch_json(query, params).await? {
            JsonValue::Array(vec) => vec
                .into_iter()
                .filter_map(|value| value.into_map_opt())
                .collect::<Vec<_>>(),
            JsonValue::Object(mut map) => {
                let data = if let Some(json_pointer) = &self.json_pointer {
                    map.pointer(json_pointer).cloned()
                } else {
                    map.remove("data").or_else(|| map.remove("result"))
                };
                if let Some(value) = data {
                    if let JsonValue::Array(vec) = value {
                        vec.into_iter()
                            .filter_map(|v| v.into_map_opt())
                            .collect::<Vec<_>>()
                    } else {
                        vec![Map::from_entry("data", value)]
                    }
                } else {
                    vec![map]
                }
            }
            _ => bail!("invalid data format"),
        };
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    async fn query_one(&self, query: &str, params: Option<&Map>) -> Result<Option<Record>, Error> {
        let record = match self.fetch_json(query, params).await? {
            JsonValue::Object(mut map) => {
                let data = if let Some(json_pointer) = &self.json_pointer {
                    map.pointer(json_pointer).cloned()
                } else {
                    map.remove("data").or_else(|| map.remove("result"))
                };
                if let Some(value) = data {
                    if let JsonValue::Object(data) = value {
                        data.into_avro_record()
                    } else {
                        Record::from_entry("data", value)
                    }
                } else {
                    map.into_avro_record()
                }
            }
            _ => bail!("invalid data format"),
        };
        Ok(Some(record))
    }

    async fn query_one_as<T: DeserializeOwned>(
        &self,
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        if let JsonValue::Object(mut map) = self.fetch_json(query, params).await? {
            let data = if let Some(json_pointer) = &self.json_pointer {
                map.pointer(json_pointer).cloned()
            } else {
                map.remove("data").or_else(|| map.remove("result"))
            };
            if let Some(value) = data {
                serde_json::from_value(value).map_err(Error::from)
            } else {
                serde_json::from_value(map.into()).map_err(Error::from)
            }
        } else {
            bail!("invalid data format");
        }
    }
}
