use axum::{
    body::Bytes,
    extract::{ConnectInfo, FromRequest, MatchedPath, OriginalUri, Request},
    http::{Method, Uri},
};
use std::{
    borrow::Cow,
    convert::Infallible,
    mem,
    net::{IpAddr, SocketAddr},
    ops::{Deref, DerefMut},
    sync::Arc,
};
use zino_core::{error::Error, extension::HeaderMapExt, state::Data};
use zino_http::request::{Context, RequestContext};

/// An HTTP request extractor.
pub struct Extractor<T>(T);

impl<T> Deref for Extractor<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Extractor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Request> for Extractor<Request> {
    #[inline]
    fn from(request: Request) -> Self {
        Self(request)
    }
}

impl From<Extractor<Request>> for Request {
    #[inline]
    fn from(extractor: Extractor<Request>) -> Self {
        extractor.0
    }
}

impl RequestContext for Extractor<Request> {
    type Method = Method;
    type Uri = Uri;

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
    fn matched_route(&self) -> Cow<'_, str> {
        if let Some(path) = self.extensions().get::<MatchedPath>() {
            path.as_str().into()
        } else {
            self.uri().path().into()
        }
    }

    #[inline]
    fn request_path(&self) -> &str {
        self.uri().path()
    }

    #[inline]
    fn get_query_string(&self) -> Option<&str> {
        self.uri().query()
    }

    #[inline]
    fn get_header(&self, name: &str) -> Option<&str> {
        self.headers().get(name)?.to_str().ok()
    }

    #[inline]
    fn client_ip(&self) -> Option<IpAddr> {
        self.headers().get_client_ip().or_else(|| {
            self.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .map(|socket| socket.ip())
        })
    }

    #[inline]
    fn get_context(&self) -> Option<Arc<Context>> {
        self.extensions().get::<Arc<Context>>().cloned()
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
    async fn read_body_bytes(&mut self) -> Result<Bytes, Error> {
        let body = mem::take(self.body_mut());
        let bytes = axum::body::to_bytes(body, usize::MAX).await?;
        Ok(bytes)
    }
}

impl FromRequest<()> for Extractor<Request> {
    type Rejection = Infallible;

    #[inline]
    async fn from_request(req: Request, _state: &()) -> Result<Self, Self::Rejection> {
        Ok(Extractor(req))
    }
}
