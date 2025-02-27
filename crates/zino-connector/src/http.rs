use super::{Connector, DataSource, DataSourceConnector::Http};
use crate::helper;
use http::{
    Method,
    header::{HeaderMap, HeaderName},
};
use reqwest::Response;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::value::RawValue;
use toml::Table;
use url::Url;
use zino_core::{
    JsonValue, Map, Record,
    application::Agent,
    bail,
    error::Error,
    extension::{
        AvroRecordExt, HeaderMapExt, JsonObjectExt, JsonValueExt, TomlTableExt, TomlValueExt,
    },
    trace::TraceContext,
    warn,
};

/// A connector to HTTP services.
///
/// # Examples
///
/// ```rust,ignore
/// use zino_connector::HttpConnector;
/// use zino_core::{error::Error, state::State, LazyLock, Map};
///
/// static AMAP_GEOCODE_CONNECTOR: LazyLock<HttpConnector> = LazyLock::new(|| {
///     let config = State::shared()
///         .get_config("amap")
///         .expect("field `amap` should be a table");
///     let base_url = "https://restapi.amap.com/v3/geocode/geo";
///     connector = HttpConnector::try_new("GET", base_url)
///         .expect("fail to construct AMap Geocode connector")
///         .query("output", "JSON")
///         .query("key", config.get_str("key"))
///         .query_param("address", None)
///         .query_param("city", None)
///         .build_query()
///         .expect("fail to build a query template for the connector")
/// });
///
/// async fn get_lng_lat(city: &str, address: &str) -> Result<(f32, f32), Error> {
///     let params = json!({
///         "city": city,
///         "address": address,
///     });
///     let data: Map = AMAP_GEOCODE_CONNECTOR
///         .fetch_json(None, params.as_object())
///         .await?;
///     if let Some(Ok(postions)) = data
///         .pointer("/geocodes/0/location")
///         .and_then(|v| v.parse_array())
///     {
///         Ok((postions[0], postions[1]))
///     } else {
///         bail!("fail to parse the location");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HttpConnector {
    /// HTTP request method (VERB).
    method: Method,
    /// Base URL.
    base_url: Url,
    /// HTTP request query.
    query: Map,
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
            query: Map::new(),
            headers: Map::new(),
            body: None,
            json_pointer: None,
        })
    }

    /// Attempts to construct a new instance from the config.
    #[inline]
    pub fn try_from_config(config: &Table) -> Result<Self, Error> {
        let method = config.get_str("method").unwrap_or("GET");
        let base_url = config
            .get_str("base-url")
            .ok_or_else(|| warn!("base URL should be specified"))?;

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

    /// Adds a key/value pair for the request query.
    #[inline]
    pub fn query(mut self, key: &str, value: impl Into<JsonValue>) -> Self {
        let value = value.into();
        if !value.is_null() {
            self.query.upsert(key, value);
        }
        self
    }

    /// Adds a parameter for the request query.
    #[inline]
    pub fn query_param(mut self, key: &str, param: Option<&str>) -> Self {
        if let Some(param) = param {
            self.query.upsert(key, ["${", param, "}"].concat());
        } else {
            self.query.upsert(key, ["${", key, "}"].concat());
        }
        self
    }

    /// Builds the request query.
    pub fn build_query(mut self) -> Result<Self, Error> {
        if !self.query.is_empty() {
            let query = serde_qs::to_string(&self.query)?;
            self.base_url.set_query(Some(&query));
            self.query.clear();
        }
        Ok(self)
    }

    /// Adds a key/value pair for the request headers.
    #[inline]
    pub fn header(mut self, key: &str, value: impl Into<JsonValue>) -> Self {
        self.headers.upsert(key, value);
        self
    }

    /// Sets the request path.
    #[inline]
    pub fn set_path(&mut self, path: &str) {
        self.base_url.set_path(path);
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
    pub async fn fetch(
        &self,
        query: Option<&str>,
        params: Option<&Map>,
    ) -> Result<Response, Error> {
        let mut url = self.base_url.clone();
        if let Some(query) = query.filter(|s| !s.is_empty()) {
            url.set_query(Some(query));
        } else if !self.query.is_empty() {
            let query = serde_qs::to_string(&self.query)?;
            url.set_query(Some(&query));
        }

        let url = percent_encoding::percent_decode_str(url.as_str()).decode_utf8()?;
        let resource = helper::format_query(&url, params);
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
        trace_context.record_trace_state();
        Agent::request_builder(resource.as_ref(), Some(&options))?
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
        query: Option<&str>,
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

        let connector = HttpConnector::try_from_config(config)?;
        let data_source = DataSource::new("http", None, name, catalog, Http(connector));
        Ok(data_source)
    }

    async fn execute(&self, query: &str, params: Option<&Map>) -> Result<Option<u64>, Error> {
        let data: JsonValue = self.fetch_json(Some(query), params).await?;
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
        let records = match self.fetch_json(Some(query), params).await? {
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
        let data = match self.fetch_json(Some(query), params).await? {
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
        let record = match self.fetch_json(Some(query), params).await? {
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
        if let JsonValue::Object(mut map) = self.fetch_json(Some(query), params).await? {
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
