use actix_web::{
    body::BoxBody,
    http::{
        header::{HeaderName, HeaderValue},
        StatusCode,
    },
    HttpRequest, HttpResponse, Responder, ResponseError,
};
use std::fmt;
use zino_core::response::{Rejection, Response, ResponseCode};

/// An HTTP response for `actix-web`.
pub struct ActixResponse<S>(Response<S>);

impl<S: ResponseCode> From<Response<S>> for ActixResponse<S> {
    #[inline]
    fn from(response: Response<S>) -> Self {
        Self(response)
    }
}

impl Responder for ActixResponse<StatusCode> {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let response = self.0;
        let mut res = match response.read_bytes() {
            Ok(data) => {
                let status_code = response
                    .status_code()
                    .try_into()
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
                let body = BoxBody::new(data);
                HttpResponse::with_body(status_code, body)
            }
            Err(err) => {
                let status_code = StatusCode::INTERNAL_SERVER_ERROR;
                let body = BoxBody::new(err.to_string());
                HttpResponse::with_body(status_code, body)
            }
        };

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

        let server_timing = response.emit();
        if let Ok(header_value) = HeaderValue::try_from(server_timing.to_string()) {
            let header_name = HeaderName::from_static("server-timing");
            res.headers_mut().insert(header_name, header_value);
        }

        res
    }
}

/// An HTTP rejection response for `actix-web`.
#[derive(Debug)]
pub struct ActixRejection {
    /// Status code.
    status_code: StatusCode,
}

impl fmt::Display for ActixRejection {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.status_code)
    }
}

impl<'a> From<Rejection<'a>> for ActixRejection {
    fn from(rejection: Rejection<'a>) -> Self {
        let status_code = rejection
            .status_code()
            .try_into()
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        Self { status_code }
    }
}

impl ResponseError for ActixRejection {
    #[inline]
    fn status_code(&self) -> StatusCode {
        self.status_code
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        HttpResponse::new(self.status_code)
    }
}
