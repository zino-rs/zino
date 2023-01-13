use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, MatchedPath, OriginalUri},
    http::{Request, Uri},
};
use hyper::body::{self, Buf, Bytes};
use serde::de::DeserializeOwned;
use std::{
    convert::Infallible,
    ops::{Deref, DerefMut},
};
use toml::value::Table;
use zino_core::{
    channel::CloudEvent,
    request::{Context, RequestContext, Validation},
    response::Rejection,
    state::State,
    Map,
};

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
        let state = self
            .extensions()
            .get::<State>()
            .expect("the request extension `State` does not exist");
        state.config()
    }

    #[inline]
    fn state_data(&self) -> &Map {
        let state = self
            .extensions()
            .get::<State>()
            .expect("the request extension `State` does not exist");
        state.data()
    }

    #[inline]
    fn state_data_mut(&mut self) -> &mut Map {
        let state = self
            .extensions_mut()
            .get_mut::<State>()
            .expect("the request extension `State` does not exist");
        state.data_mut()
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

    fn matched_path(&self) -> &str {
        // The `MatchedPath` extension is always accessible on handlers added via `Router::route`,
        // but it is not accessible in middleware on nested routes.
        if let Some(path) = self.extensions().get::<MatchedPath>() {
            path.as_str()
        } else {
            self.uri().path()
        }
    }

    fn original_uri(&self) -> &Uri {
        // The `OriginalUri` extension will always be present if using
        // `Router` unless another extractor or middleware has removed it.
        if let Some(original_uri) = self.extensions().get::<OriginalUri>() {
            &original_uri.0
        } else {
            self.uri()
        }
    }

    async fn to_bytes(&mut self) -> Result<Bytes, Validation> {
        body::to_bytes(self.body_mut()).await.map_err(|err| {
            let mut validation = Validation::new();
            validation.record_fail("body", err.to_string());
            validation
        })
    }

    fn try_send(&self, message: impl Into<CloudEvent>) -> Result<(), Rejection> {
        let event = message.into();
        crate::channel::axum_channel::MessageChannel::shared()
            .try_send(event)
            .map_err(Rejection::internal_server_error)
    }

    fn parse_query<T>(&self) -> Result<T, Validation>
    where
        T: Default + DeserializeOwned + Send + 'static,
    {
        match self.uri().query() {
            Some(query) => serde_qs::from_str::<T>(query).map_err(|err| {
                let mut validation = Validation::new();
                validation.record_fail("query", err.to_string());
                validation
            }),
            None => Ok(T::default()),
        }
    }

    async fn parse_body<T>(&mut self) -> Result<T, Validation>
    where
        T: DeserializeOwned + Send + 'static,
    {
        let form_urlencoded = self
            .get_header("content-type")
            .map(|t| t.starts_with("application/x-www-form-urlencoded"))
            .unwrap_or(true);
        let body = self.body_mut();
        let result = if form_urlencoded {
            body::aggregate(body)
                .await
                .map_err(|err| err.to_string())
                .and_then(|buf| {
                    serde_urlencoded::from_reader(buf.reader()).map_err(|err| err.to_string())
                })
        } else {
            body::aggregate(body)
                .await
                .map_err(|err| err.to_string())
                .and_then(|buf| {
                    serde_json::from_reader(buf.reader()).map_err(|err| err.to_string())
                })
        };
        result.map_err(|err| {
            let mut validation = Validation::new();
            validation.record_fail("body", err);
            validation
        })
    }
}

#[async_trait]
impl FromRequest<(), Body> for AxumExtractor<Request<Body>> {
    type Rejection = Infallible;

    #[inline]
    async fn from_request(req: Request<Body>, _state: &()) -> Result<Self, Self::Rejection> {
        Ok(AxumExtractor(req))
    }
}
