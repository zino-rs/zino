use crate::response::ntex_response::NtexRejection;
use ntex::{
    http::{header::HeaderMap, Method, Payload},
    util::Bytes,
    web::{
        error::{DefaultError, ErrorRenderer},
        FromRequest, HttpRequest, WebRequest,
    },
};
use std::{
    borrow::Cow,
    convert::Infallible,
    net::IpAddr,
    ops::{Deref, DerefMut},
};
use zino_core::{
    error::Error,
    request::{Context, RequestContext, Uri},
    response::Rejection,
    state::Data,
};

/// An HTTP request extractor for `ntex`.
pub struct NtexExtractor<T>(T, Payload);

impl<T> Deref for NtexExtractor<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for NtexExtractor<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl RequestContext for NtexExtractor<HttpRequest> {
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
        self.match_info().path().into()
    }

    #[inline]
    fn header_map(&self) -> &Self::Headers {
        self.headers()
    }

    #[inline]
    fn get_header(&self, name: &str) -> Option<&str> {
        self.headers().get(name)?.to_str().ok()
    }

    #[inline]
    fn client_ip(&self) -> Option<IpAddr> {
        self.connection_info().remote().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_context(&self) -> Option<Context> {
        let extensions = self.extensions();
        extensions.get::<Context>().cloned()
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
    async fn read_body_bytes(&mut self) -> Result<Vec<u8>, Error> {
        let bytes =
            <Bytes as FromRequest<DefaultError>>::from_request(&self.0, &mut self.1).await?;
        Ok(bytes.to_vec())
    }
}

impl<Err: ErrorRenderer> From<WebRequest<Err>> for NtexExtractor<HttpRequest> {
    #[inline]
    fn from(request: WebRequest<Err>) -> Self {
        let (request, payload) = request.into_parts();
        Self(request, payload)
    }
}

impl<Err: ErrorRenderer> TryFrom<NtexExtractor<HttpRequest>> for WebRequest<Err> {
    type Error = NtexRejection;

    #[inline]
    fn try_from(extractor: NtexExtractor<HttpRequest>) -> Result<Self, Self::Error> {
        Self::from_parts(extractor.0, extractor.1).map_err(|_| {
            let error = Error::new("fail to re-constructed `WebRequest`");
            Rejection::internal_server_error(error).into()
        })
    }
}

impl From<HttpRequest> for NtexExtractor<HttpRequest> {
    #[inline]
    fn from(request: HttpRequest) -> Self {
        Self(request, Payload::None)
    }
}

impl From<NtexExtractor<HttpRequest>> for HttpRequest {
    #[inline]
    fn from(extractor: NtexExtractor<HttpRequest>) -> Self {
        extractor.0
    }
}

impl FromRequest<DefaultError> for NtexExtractor<HttpRequest> {
    type Error = Infallible;

    #[inline]
    async fn from_request(req: &HttpRequest, payload: &mut Payload) -> Result<Self, Self::Error> {
        Ok(NtexExtractor(req.clone(), payload.take()))
    }
}
