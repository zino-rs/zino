use async_trait::async_trait;
use axum::{
    body::{Body, HttpBody},
    extract::{ConnectInfo, FromRequest, MatchedPath, OriginalUri},
    http::{HeaderMap, Method, Request},
};
use bytes::{Buf, BufMut};
use std::{
    borrow::Cow,
    convert::Infallible,
    marker::Unpin,
    net::{IpAddr, SocketAddr},
    ops::{Deref, DerefMut},
    pin::Pin,
};
use zino_core::{
    error::Error,
    extension::HeaderMapExt,
    request::{Context, RequestContext, Uri},
    state::Data,
};

/// An HTTP request extractor for `axum`.
pub struct AxumExtractor<T>(T);

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

impl From<Request<Body>> for AxumExtractor<Request<Body>> {
    #[inline]
    fn from(request: Request<Body>) -> Self {
        Self(request)
    }
}

impl From<AxumExtractor<Request<Body>>> for Request<Body> {
    #[inline]
    fn from(extractor: AxumExtractor<Request<Body>>) -> Self {
        extractor.0
    }
}

impl RequestContext for AxumExtractor<Request<Body>> {
    type Method = Method;
    type Headers = HeaderMap;

    #[inline]
    fn request_method(&self) -> &Self::Method {
        self.method()
    }

    #[inline]
    fn original_uri(&self) -> &Uri {
        // The `OriginalUri` extension will always be present if using
        // `Router` unless another extractor or middleware has removed it.
        if let Some(original_uri) = self.extensions().get::<OriginalUri>() {
            &original_uri.0
        } else {
            self.uri()
        }
    }

    #[inline]
    fn header_map(&self) -> &Self::Headers {
        self.headers()
    }

    #[inline]
    fn matched_route(&self) -> Cow<'_, str> {
        if let Some(path) = self.extensions().get::<MatchedPath>() {
            path.as_str().into()
        } else {
            self.uri().path().into()
        }
    }

    #[inline]
    fn get_header(&self, name: &str) -> Option<&str> {
        self.headers().get(name)?.to_str().ok()
    }

    #[inline]
    fn get_context(&self) -> Option<Context> {
        self.extensions().get::<Context>().cloned()
    }

    #[inline]
    fn get_data<T: Clone + Send + Sync + 'static>(&self) -> Option<T> {
        self.extensions().get::<Data<T>>().map(|data| data.get())
    }

    #[inline]
    fn set_data<T: Clone + Send + Sync + 'static>(&mut self, value: T) -> Option<T> {
        self.extensions_mut()
            .insert(Data::new(value))
            .map(|data| data.into_inner())
    }

    #[inline]
    fn client_ip(&self) -> Option<IpAddr> {
        self.header_map().get_client_ip().or_else(|| {
            self.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|socket| socket.ip())
        })
    }

    #[inline]
    async fn read_body_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let bytes = to_bytes(self.body_mut()).await?;
        Ok(bytes)
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

/// Concatenates the buffers from a body into a single `Bytes` asynchronously.
///
/// Copy from https://docs.rs/hyper/0.14.27/hyper/body/fn.to_bytes.html
async fn to_bytes<T: HttpBody + Unpin>(mut body: T) -> Result<Vec<u8>, T::Error> {
    let _ = Pin::new(&mut body);

    // If there's only 1 chunk, we can just return Buf::to_bytes()
    let mut first = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(Vec::new());
    };

    let second = if let Some(buf) = body.data().await {
        buf?
    } else {
        return Ok(first.copy_to_bytes(first.remaining()).into());
    };

    // Don't pre-emptively reserve *too* much.
    let rest = (body.size_hint().lower() as usize).min(1024 * 16);
    let cap = first
        .remaining()
        .saturating_add(second.remaining())
        .saturating_add(rest);

    // With more than 1 buf, we gotta flatten into a Vec first.
    let mut vec = Vec::with_capacity(cap);
    vec.put(first);
    vec.put(second);

    while let Some(buf) = body.data().await {
        vec.put(buf?);
    }

    Ok(vec)
}
