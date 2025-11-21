use crate::SharedString;

/// An interface for application code.
pub trait ApplicationCode {
    /// An integer code for the application.
    fn code(&self) -> i32;

    /// A descriptive message.
    fn message(&self) -> SharedString;
}
