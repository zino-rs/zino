use async_trait::async_trait;
use axum::{
    body::{Body, Bytes},
    extract::{FromRequest, MatchedPath, OriginalUri},
    http::{HeaderMap, Method, Request, Uri},
};
use std::{
    borrow::Cow,
    convert::Infallible,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};
use tower_cookies::{Cookie, Cookies, Key};
use zino_core::{
    application::Application,
    error::Error,
    request::{Context, RequestContext},
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
        self.headers()
            .get(name)?
            .to_str()
            .inspect_err(|err| tracing::error!("{err}"))
            .ok()
    }

    #[inline]
    fn get_context(&self) -> Option<Context> {
        self.extensions().get::<Context>().cloned()
    }

    #[inline]
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        let cookies = self.extensions().get::<Cookies>()?;
        let key = LazyLock::force(&COOKIE_PRIVATE_KEY);
        let signed_cookies = cookies.signed(key);
        signed_cookies.get(name)
    }

    #[inline]
    async fn read_body_bytes(&mut self) -> Result<Bytes, Error> {
        let bytes = hyper::body::to_bytes(self.body_mut()).await?;
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

/// Private key for cookie signing.
static COOKIE_PRIVATE_KEY: LazyLock<Key> = LazyLock::new(|| {
    let secret_key = crate::Cluster::secret_key();
    Key::try_from(secret_key).unwrap_or_else(|_| Key::generate())
});
