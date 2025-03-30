use ntex::{
    http::{
        ResponseError, StatusCode,
        body::Body,
        header::{self, HeaderName, HeaderValue},
    },
    web::{HttpRequest, HttpResponse, Responder, WebResponseError},
};
use std::fmt;
use zino_http::{
    response::{Rejection, Response, ResponseCode},
    timing::TimingMetric,
};

/// An HTTP response for `ntex`.
pub struct NtexResponse<S: ResponseCode = StatusCode>(Response<S>);

impl<S: ResponseCode> From<Response<S>> for NtexResponse<S> {
    #[inline]
    fn from(response: Response<S>) -> Self {
        Self(response)
    }
}

impl<S: ResponseCode> Responder for NtexResponse<S> {
    async fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        let mut response = self.0;
        if !response.has_context() {
            let req = crate::Request::from(req.to_owned());
            response = response.context(&req);
        }

        let mut res = build_http_response(&mut response);
        for (name, value) in response.finalize() {
            if let Some(Ok(header_name)) = name.map(|name| HeaderName::try_from(name.as_str())) {
                if let Ok(header_value) = HeaderValue::try_from(value) {
                    res.headers_mut().insert(header_name, header_value);
                }
            }
        }

        res
    }
}

/// An HTTP rejection response for `ntex`.
pub struct NtexRejection(Response<StatusCode>);

impl fmt::Debug for NtexRejection {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.0.message().unwrap_or("OK"))
    }
}

impl fmt::Display for NtexRejection {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.status_code().fmt(f)
    }
}

impl From<Rejection> for NtexRejection {
    #[inline]
    fn from(rejection: Rejection) -> Self {
        Self(Response::from(rejection))
    }
}

impl ResponseError for NtexRejection {
    fn error_response(&self) -> HttpResponse {
        let mut response = self.0.to_owned();
        let mut res = build_http_response(&mut response);
        let request_id = response.request_id();
        if !request_id.is_nil() {
            if let Ok(header_value) = HeaderValue::try_from(request_id.to_string()) {
                let header_name = HeaderName::from_static("x-request-id");
                res.headers_mut().insert(header_name, header_value);
            }
        }

        let (traceparent, tracestate) = response.trace_context();
        if let Ok(header_value) = HeaderValue::try_from(traceparent) {
            let header_name = HeaderName::from_static("traceparent");
            res.headers_mut().insert(header_name, header_value);
        }
        if let Ok(header_value) = HeaderValue::try_from(tracestate) {
            let header_name = HeaderName::from_static("tracestate");
            res.headers_mut().insert(header_name, header_value);
        }

        let response_time = response.response_time();
        let timing = TimingMetric::new("total".into(), None, response_time.into());
        if let Ok(header_value) = HeaderValue::try_from(timing.to_string()) {
            let header_name = HeaderName::from_static("server-timing");
            res.headers_mut().insert(header_name, header_value);
        }

        for (name, value) in response.headers() {
            if let Ok(header_name) = HeaderName::try_from(name.as_str()) {
                if let Ok(header_value) = HeaderValue::try_from(value) {
                    res.headers_mut().insert(header_name, header_value);
                }
            }
        }

        res
    }
}

impl WebResponseError for NtexRejection {
    #[inline]
    fn error_response(&self, _: &HttpRequest) -> HttpResponse {
        ResponseError::error_response(&self)
    }
}

/// Build http response from `zino_core::response::Response`.
fn build_http_response<S: ResponseCode>(response: &mut Response<S>) -> HttpResponse {
    match response.read_bytes() {
        Ok(data) => {
            let status_code = response
                .status_code()
                .try_into()
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            let body = Body::from(data.to_vec());
            let mut res = HttpResponse::with_body(status_code, body);
            if let Ok(header_value) = HeaderValue::try_from(response.content_type()) {
                res.headers_mut().insert(header::CONTENT_TYPE, header_value);
            }
            res
        }
        Err(err) => {
            let status_code = StatusCode::INTERNAL_SERVER_ERROR;
            let body = Body::from(err.to_string());
            let mut res = HttpResponse::with_body(status_code, body);
            res.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            res
        }
    }
}
