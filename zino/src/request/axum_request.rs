use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, OriginalUri, Path},
    http::{Request, Uri},
    RequestExt,
};
use hyper::body::Buf;
use serde::de::DeserializeOwned;
use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use toml::value::Table;
use zino_core::{CloudEvent, Context, Map, Rejection, RequestContext, State, Validation};

/// An HTTP request extractor for `axum`.
pub struct AxumExtractor<T>(pub(crate) T);

impl<T> Deref for AxumExtractor<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for AxumExtractor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RequestContext for AxumExtractor<Request<Body>> {
    #[inline]
    fn config(&self) -> &Table {
        let state = self.extensions().get::<Arc<State>>().unwrap();
        state.config()
    }

    #[inline]
    fn get_context(&self) -> Option<&Context> {
        self.extensions().get::<Context>()
    }

    #[inline]
    fn get_header(&self, key: &str) -> Option<&str> {
        self.headers().get(key)?.to_str().ok()
    }

    #[inline]
    fn request_method(&self) -> &str {
        self.method().as_str()
    }

    #[inline]
    fn parse_query(&self) -> Result<Map, Validation> {
        match self.uri().query() {
            Some(query) => serde_qs::from_str::<Map>(query).map_err(|err| {
                let mut validation = Validation::new();
                validation.record_fail("query", err.to_string());
                validation
            }),
            None => Ok(Map::new()),
        }
    }

    async fn parse_body(&mut self) -> Result<Map, Validation> {
        let form_urlencoded = self
            .get_header("content-type")
            .unwrap_or("application/x-www-form-urlencoded")
            .starts_with("application/x-www-form-urlencoded");
        let body = self.body_mut();
        let result = if form_urlencoded {
            hyper::body::aggregate(body)
                .await
                .map_err(|err| err.to_string())
                .and_then(|buf| {
                    serde_urlencoded::from_reader(buf.reader()).map_err(|err| err.to_string())
                })
        } else {
            hyper::body::aggregate(body)
                .await
                .map_err(|err| err.to_string())
                .and_then(|buf| {
                    serde_json::from_reader(buf.reader()).map_err(|err| err.to_string())
                })
        };
        result.map_err(|err| {
            let mut validation = Validation::new();
            validation.record_fail("query", err);
            validation
        })
    }

    fn try_send(&self, message: impl Into<CloudEvent>) -> Result<(), Rejection> {
        let event = message.into();
        crate::channel::axum_channel::MessageChannel::shared()
            .try_send(event)
            .map_err(Rejection::internal_server_error)
    }
}

#[async_trait]
impl FromRequest<(), Body> for AxumExtractor<Request<Body>> {
    type Rejection = Infallible;

    async fn from_request(req: Request<Body>, _state: &()) -> Result<Self, Self::Rejection> {
        Ok(AxumExtractor(req))
    }
}

impl AxumExtractor<Request<Body>> {
    /// Extracts the original request URI regardless of nesting.
    pub async fn original_uri(&mut self) -> Uri {
        let OriginalUri(uri) = self.extract_parts::<OriginalUri>().await.unwrap();
        uri
    }

    /// Parses the route params as an instance of type `T`.
    pub async fn parse_params<T>(&mut self) -> Result<T, Rejection>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let Path(param) = self.extract_parts::<Path<T>>().await.map_err(|err| {
            let mut validation = Validation::new();
            validation.record_fail("params", err.to_string());
            Rejection::bad_request(validation)
        })?;
        Ok(param)
    }
}
