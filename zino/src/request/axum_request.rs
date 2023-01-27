use async_trait::async_trait;
use axum::{
    body::Body,
    extract::{FromRequest, MatchedPath},
    http::{Request, Uri},
};
use hyper::body::{self, Buf};
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
        self.headers()
            .get(key)?
            .to_str()
            .inspect_err(|err| tracing::error!("{err}"))
            .ok()
    }

    #[inline]
    fn request_method(&self) -> &str {
        self.method().as_str()
    }

    #[inline]
    fn matched_route(&self) -> &str {
        if let Some(path) = self.extensions().get::<MatchedPath>() {
            path.as_str()
        } else {
            self.uri().path()
        }
    }

    #[inline]
    fn original_uri(&self) -> &Uri {
        self.uri()
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

    #[inline]
    fn try_send(&self, message: impl Into<CloudEvent>) -> Result<(), Rejection> {
        crate::channel::axum_channel::MessageChannel::shared()
            .try_send(message.into())
            .map_err(Rejection::internal_server_error)
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
