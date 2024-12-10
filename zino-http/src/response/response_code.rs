use serde::Serialize;
use std::borrow::Cow;
use zino_core::SharedString;

/// Trait for response code.
/// See [Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807).
pub trait ResponseCode {
    /// A type for the error code.
    type ErrorCode: Serialize;

    /// A type for the business code.
    type BusinessCode: Serialize;

    /// 200 Ok.
    const OK: Self;
    /// 400 Bad Request.
    const BAD_REQUEST: Self;
    /// 500 Internal Server Error.
    const INTERNAL_SERVER_ERROR: Self;

    /// Status code.
    fn status_code(&self) -> u16;

    /// Returns `true` if the response is successful.
    fn is_success(&self) -> bool;

    /// Error code.
    #[inline]
    fn error_code(&self) -> Option<Self::ErrorCode> {
        None
    }

    /// Business code.
    #[inline]
    fn business_code(&self) -> Option<Self::BusinessCode> {
        None
    }

    /// A URI reference that identifies the problem type.
    /// For successful response, it should be `None`.
    fn type_uri(&self) -> Option<SharedString> {
        None
    }

    /// A short, human-readable summary of the problem type.
    /// For successful response, it should be `None`.
    fn title(&self) -> Option<SharedString> {
        None
    }

    /// A context-specific descriptive message. If the response is not successful,
    /// it should be a human-readable explanation specific to this occurrence of the problem.
    fn message(&self) -> Option<SharedString> {
        None
    }
}

macro_rules! impl_response_code {
    ($Ty:ty) => {
        impl ResponseCode for $Ty {
            type ErrorCode = SharedString;
            type BusinessCode = u16;

            const OK: Self = Self::OK;
            const BAD_REQUEST: Self = Self::BAD_REQUEST;
            const INTERNAL_SERVER_ERROR: Self = Self::INTERNAL_SERVER_ERROR;

            #[inline]
            fn status_code(&self) -> u16 {
                self.as_u16()
            }

            #[inline]
            fn is_success(&self) -> bool {
                self.is_success()
            }

            #[inline]
            fn type_uri(&self) -> Option<SharedString> {
                None
            }

            #[inline]
            fn title(&self) -> Option<SharedString> {
                if self.is_success() {
                    None
                } else {
                    self.canonical_reason().map(Cow::Borrowed)
                }
            }

            #[inline]
            fn message(&self) -> Option<SharedString> {
                if self.is_success() {
                    self.canonical_reason().map(Cow::Borrowed)
                } else {
                    None
                }
            }
        }
    };
}

impl_response_code!(http::StatusCode);

#[cfg(feature = "http02")]
impl_response_code!(http02::StatusCode);
