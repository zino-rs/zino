use crate::SharedString;
use http::StatusCode;
use std::borrow::Cow;

/// Response code.
/// See [Problem Details for HTTP APIs](https://tools.ietf.org/html/rfc7807).
pub trait ResponseCode {
    /// 200 Ok.
    const OK: Self;

    /// Status code.
    fn status_code(&self) -> u16;

    /// Error code.
    fn error_code(&self) -> Option<SharedString>;

    /// Returns `true` if the response is successful.
    fn is_success(&self) -> bool;

    /// A URI reference that identifies the problem type.
    /// For successful response, it should be `None`.
    fn type_uri(&self) -> Option<SharedString>;

    /// A short, human-readable summary of the problem type.
    /// For successful response, it should be `None`.
    fn title(&self) -> Option<SharedString>;

    /// A context-specific descriptive message. If the response is not successful,
    /// it should be a human-readable explanation specific to this occurrence of the problem.
    fn message(&self) -> Option<SharedString>;
}

impl ResponseCode for StatusCode {
    const OK: Self = StatusCode::OK;

    #[inline]
    fn status_code(&self) -> u16 {
        self.as_u16()
    }

    #[inline]
    fn error_code(&self) -> Option<SharedString> {
        None
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
