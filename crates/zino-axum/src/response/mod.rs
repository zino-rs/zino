use axum::{
    body::Body,
    http::{
        StatusCode,
        header::{self, HeaderValue},
    },
    response::IntoResponse,
};
use zino_http::response::{Rejection, Response, ResponseCode};

/// An HTTP response for `axum`.
pub struct AxumResponse<S: ResponseCode = StatusCode>(Response<S>);

impl<S: ResponseCode> From<Response<S>> for AxumResponse<S> {
    #[inline]
    fn from(response: Response<S>) -> Self {
        Self(response)
    }
}

impl<S: ResponseCode> IntoResponse for AxumResponse<S> {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        build_http_response(self.0)
    }
}

/// An HTTP rejection response for `axum`.
pub struct AxumRejection(Response<StatusCode>);

impl From<Rejection> for AxumRejection {
    #[inline]
    fn from(rejection: Rejection) -> Self {
        Self(rejection.into())
    }
}

impl IntoResponse for AxumRejection {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        build_http_response(self.0)
    }
}

/// Build http response from `zino_core::response::Response`.
pub(crate) fn build_http_response<S: ResponseCode>(
    mut response: Response<S>,
) -> axum::response::Response {
    let mut res = match response.read_bytes() {
        Ok(data) => axum::response::Response::builder()
            .status(response.status_code())
            .header(header::CONTENT_TYPE, response.content_type())
            .body(Body::from(data))
            .unwrap_or_default(),
        Err(err) => axum::response::Response::builder()
            .status(S::INTERNAL_SERVER_ERROR.status_code())
            .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
            .body(Body::from(err.to_string()))
            .unwrap_or_default(),
    };

    for (name, value) in response.finalize() {
        if let Some(header_name) = name {
            if let Ok(header_value) = HeaderValue::try_from(value) {
                res.headers_mut().insert(header_name, header_value);
            }
        }
    }

    res
}
