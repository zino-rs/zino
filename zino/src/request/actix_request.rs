use actix_web::{
    dev::Payload,
    http::{header::HeaderMap, Method},
    web::Bytes,
    FromRequest, HttpMessage, HttpRequest,
};
use std::{
    cell::{Ref, RefMut},
    convert::Infallible,
    future,
    ops::{Deref, DerefMut},
    sync::LazyLock,
};
use toml::value::Table;
use tower_cookies::{Cookie, Cookies, Key};
use zino_core::{
    application::Application,
    error::Error,
    request::{Context, RequestContext},
    state::State,
    Map,
};

/// An HTTP request extractor for `actix-web`.
pub struct ActixExtractor<T>(T, Payload);

impl<T> Deref for ActixExtractor<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ActixExtractor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RequestContext for ActixExtractor<HttpRequest> {
    type Method = Method;
    type Headers = HeaderMap;

    #[inline]
    fn request_method(&self) -> &Self::Method {
        self.method()
    }

    #[inline]
    fn request_path(&self) -> &str {
        self.uri().path()
    }

    #[inline]
    fn header_map(&self) -> &Self::Headers {
        self.headers().into()
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
    fn get_query_string(&self) -> Option<&str> {
        self.uri().query()
    }

    #[inline]
    fn get_context(&self) -> Option<&Context> {
        let extensions = Ref::leak(self.extensions());
        extensions.get::<Context>()
    }

    #[inline]
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        let extensions = Ref::leak(self.extensions());
        let cookies = extensions.get::<Cookies>()?;
        let key = LazyLock::force(&COOKIE_PRIVATE_KEY);
        let signed_cookies = cookies.signed(key);
        signed_cookies.get(name)
    }

    #[inline]
    fn add_cookie(&self, cookie: Cookie<'static>) {
        self.extensions().get::<Cookies>().map(|cookies| {
            let key = LazyLock::force(&COOKIE_PRIVATE_KEY);
            let signed_cookies = cookies.signed(key);
            signed_cookies.add(cookie);
        });
    }

    #[inline]
    fn matched_route(&self) -> String {
        if let Some(path) = self.match_pattern() {
            path
        } else {
            self.uri().path().to_owned()
        }
    }

    #[inline]
    fn config(&self) -> &Table {
        let extensions = Ref::leak(self.extensions());
        let state = extensions
            .get::<State>()
            .expect("the request extension `State` does not exist");
        state.config()
    }

    #[inline]
    fn state_data(&self) -> &Map {
        let extensions = Ref::leak(self.extensions());
        let state = extensions
            .get::<State>()
            .expect("the request extension `State` does not exist");
        state.data()
    }

    #[inline]
    fn state_data_mut(&mut self) -> &mut Map {
        let extensions = RefMut::leak(self.extensions_mut());
        let state = extensions
            .get_mut::<State>()
            .expect("the request extension `State` does not exist");
        state.data_mut()
    }

    #[inline]
    async fn read_body_bytes(&mut self) -> Result<Bytes, Error> {
        let bytes = Bytes::from_request(&self.0, &mut self.1).await?;
        Ok(bytes)
    }
}

impl From<ActixExtractor<HttpRequest>> for HttpRequest {
    #[inline]
    fn from(extractor: ActixExtractor<HttpRequest>) -> Self {
        extractor.0
    }
}

impl FromRequest for ActixExtractor<HttpRequest> {
    type Error = Infallible;
    type Future = future::Ready<Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        future::ready(Ok(ActixExtractor(req.clone(), payload.take())))
    }
}

/// Private key for cookie signing.
static COOKIE_PRIVATE_KEY: LazyLock<Key> = LazyLock::new(|| {
    let secret_key = crate::Cluster::secret_key();
    Key::try_from(secret_key).unwrap_or_else(|_| Key::generate())
});
