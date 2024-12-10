use crate::helper;
use http::{
    header::{HeaderMap, HeaderName},
    Method,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::value::RawValue;
use toml::Table;
use url::Url;
use zino_core::{
    application::http_client,
    bail,
    error::Error,
    extension::{HeaderMapExt, JsonObjectExt, JsonValueExt, TomlTableExt, TomlValueExt},
    trace::TraceContext,
    JsonValue, Map,
};

/// User-defined HTTP callbacks.
pub struct WebHook {
    /// Webhook name.
    name: String,
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
    /// Optional request params.
    params: Option<Map>,
}

impl WebHook {
    /// Attempts to construct a new instance from the config.
    pub fn try_new(config: &Table) -> Result<Self, Error> {
        let name = config.get_str("name").unwrap_or("webhook");
        let method = if let Some(method) = config.get_str("method") {
            method.parse()?
        } else {
            Method::GET
        };
        let mut base_url = if let Some(base_url) = config.get_str("base-url") {
            base_url.parse::<Url>()?
        } else {
            bail!("the base URL should be specified");
        };
        if let Some(query) = config.get_table("query") {
            let query = serde_qs::to_string(query)?;
            base_url.set_query(Some(&query));
        }

        let headers = config
            .get("headers")
            .and_then(|v| v.to_json_value().into_map_opt())
            .unwrap_or_default();
        let body = if let Some(body) = config.get_table("body") {
            Some(serde_json::value::to_raw_value(body)?)
        } else {
            None
        };
        let params = config
            .get("params")
            .and_then(|v| v.to_json_value().into_map_opt());
        Ok(Self {
            name: name.to_owned(),
            method,
            base_url,
            query: Map::new(),
            headers,
            body,
            params,
        })
    }

    /// Gets a webhook with the specific name from the OpenAPI docs.
    #[cfg(feature = "openapi")]
    #[inline]
    pub fn get_from_openapi(name: &str) -> Option<&'static WebHook> {
        crate::openapi::get_webhook(name)
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
            self.query.upsert(key, format!("${param}"));
        } else {
            self.query.upsert(key, format!("${key}"));
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

    /// Sets the request params.
    #[inline]
    pub fn set_params(&mut self, params: impl Into<JsonValue>) {
        self.params = params.into().into_map_opt();
    }

    /// Returns the webhook name.
    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Triggers the webhook and deserializes the response body via JSON.
    pub async fn trigger<T: DeserializeOwned>(&self) -> Result<T, Error> {
        let params = self.params.as_ref();
        let resource = helper::format_query(self.base_url.as_str(), params);
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

        let response = http_client::request_builder(resource.as_ref(), Some(&options))?
            .headers(headers)
            .header("traceparent", trace_context.traceparent())
            .header("tracestate", trace_context.tracestate())
            .send()
            .await?
            .error_for_status()?;
        let data = if response.headers().has_json_content_type() {
            response.json().await?
        } else {
            let text = response.text().await?;
            serde_json::from_str(&text)?
        };
        Ok(data)
    }
}
