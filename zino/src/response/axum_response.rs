use axum::response::IntoResponse;
use zino_core::response::{FullResponse, Rejection, Response, ResponseCode};

/// An HTTP response for `axum`.
pub struct AxumResponse<S>(Response<S>);

impl<S: ResponseCode> From<Response<S>> for AxumResponse<S> {
    #[inline]
    fn from(response: Response<S>) -> Self {
        Self(response)
    }
}

impl<S: ResponseCode> IntoResponse for AxumResponse<S> {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        FullResponse::from(self.0).into_response()
    }
}

/// An HTTP rejection response for `axum`.
pub struct AxumRejection(FullResponse);

impl From<Rejection> for AxumRejection {
    #[inline]
    fn from(rejection: Rejection) -> Self {
        Self(rejection.into())
    }
}

impl IntoResponse for AxumRejection {
    #[inline]
    fn into_response(self) -> axum::response::Response {
        FullResponse::from(self.0).into_response()
    }
}
