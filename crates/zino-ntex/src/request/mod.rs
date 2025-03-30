use crate::response::NtexRejection;
use bytes::Bytes;
use ntex::{
    http::{Method, Payload, Uri},
    util,
    web::{
        FromRequest, HttpRequest, WebRequest,
        error::{DefaultError, ErrorRenderer},
    },
};
use std::{
    borrow::Cow,
    convert::Infallible,
    net::IpAddr,
    ops::{Deref, DerefMut},
    sync::Arc,
};
use zino_core::{error::Error, state::Data};
use zino_http::{
    request::{Context, RequestContext},
    response::Rejection,
};

/// An HTTP request extractor.
pub struct Extractor<T>(T, Payload);

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

impl RequestContext for Extractor<HttpRequest> {
    type Method = Method;
    type Uri = Uri;

    #[inline]
    fn request_method(&self) -> &Self::Method {
        self.method()
    }

    #[inline]
    fn original_uri(&self) -> &Self::Uri {
        self.uri()
    }

    #[inline]
    fn matched_route(&self) -> Cow<'_, str> {
        self.match_info().path().into()
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
        self.connection_info().remote().and_then(|s| s.parse().ok())
    }

    #[inline]
    fn get_context(&self) -> Option<Arc<Context>> {
        let extensions = self.extensions();
        extensions.get::<Arc<Context>>().cloned()
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
        let bytes =
            <util::Bytes as FromRequest<DefaultError>>::from_request(&self.0, &mut self.1).await?;
        Ok(bytes.to_vec().into())
    }
}

impl<Err: ErrorRenderer> From<WebRequest<Err>> for Extractor<HttpRequest> {
    #[inline]
    fn from(request: WebRequest<Err>) -> Self {
        let (request, payload) = request.into_parts();
        Self(request, payload)
    }
}

impl<Err: ErrorRenderer> TryFrom<Extractor<HttpRequest>> for WebRequest<Err> {
    type Error = NtexRejection;

    #[inline]
    fn try_from(extractor: Extractor<HttpRequest>) -> Result<Self, Self::Error> {
        Self::from_parts(extractor.0, extractor.1).map_err(|_| {
            let error = Error::new("fail to re-constructed `WebRequest`");
            Rejection::internal_server_error(error).into()
        })
    }
}

impl From<HttpRequest> for Extractor<HttpRequest> {
    #[inline]
    fn from(request: HttpRequest) -> Self {
        Self(request, Payload::None)
    }
}

impl From<Extractor<HttpRequest>> for HttpRequest {
    #[inline]
    fn from(extractor: Extractor<HttpRequest>) -> Self {
        extractor.0
    }
}

impl FromRequest<DefaultError> for Extractor<HttpRequest> {
    type Error = Infallible;

    #[inline]
    async fn from_request(req: &HttpRequest, payload: &mut Payload) -> Result<Self, Self::Error> {
        Ok(Extractor(req.to_owned(), payload.take()))
    }
}
