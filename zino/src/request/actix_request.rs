use actix_web::{
    cookie::Cookie,
    dev::{Payload, ServiceRequest},
    http::{header::HeaderMap, Method, Uri},
    web::Bytes,
    FromRequest, HttpMessage, HttpRequest,
};
use std::{
    borrow::Cow,
    convert::Infallible,
    future,
    net::IpAddr,
    ops::{Deref, DerefMut},
};
use zino_core::{
    error::Error,
    request::{Context, RequestContext},
    state::Data,
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
    fn original_uri(&self) -> &Uri {
        self.uri()
    }

    #[inline]
    fn matched_route(&self) -> Cow<'_, str> {
        if let Some(path) = self.match_pattern() {
            path.into()
        } else {
            self.uri().path().into()
        }
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
    fn get_context(&self) -> Option<Context> {
        let extensions = self.extensions();
        extensions.get::<Context>().cloned()
    }

    #[inline]
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        self.cookie(name)
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
        self.connection_info()
            .realip_remote_addr()
            .and_then(|s| s.parse().ok())
    }

    #[inline]
    async fn read_body_bytes(&mut self) -> Result<Bytes, Error> {
        let bytes = Bytes::from_request(&self.0, &mut self.1).await?;
        Ok(bytes)
    }
}

impl From<ServiceRequest> for ActixExtractor<HttpRequest> {
    #[inline]
    fn from(request: ServiceRequest) -> Self {
        let (req, payload) = request.into_parts();
        Self(req, payload)
    }
}

impl From<ActixExtractor<HttpRequest>> for ServiceRequest {
    #[inline]
    fn from(extractor: ActixExtractor<HttpRequest>) -> Self {
        ServiceRequest::from_parts(extractor.0, extractor.1)
    }
}

impl From<HttpRequest> for ActixExtractor<HttpRequest> {
    #[inline]
    fn from(request: HttpRequest) -> Self {
        Self(request, Payload::None)
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
