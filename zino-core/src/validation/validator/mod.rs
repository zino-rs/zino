//! Common validation rules.

mod date;
mod date_time;
mod email;
mod host;
mod hostname;
mod ip_addr;
mod ipv4_addr;
mod ipv6_addr;
mod time;
mod uri;
mod uuid;

pub use date::DateValidator;
pub use date_time::DateTimeValidator;
pub use email::EmailValidator;
pub use host::HostValidator;
pub use hostname::HostnameValidator;
pub use ip_addr::IpAddrValidator;
pub use ipv4_addr::Ipv4AddrValidator;
pub use ipv6_addr::Ipv6AddrValidator;
pub use time::TimeValidator;
pub use uri::UriValidator;
pub use uuid::UuidValidator;

/// A generic validator.
pub trait Validator<T: ?Sized> {
    /// The error type.
    type Error: std::error::Error;

    /// Validates the data.
    fn validate(&self, data: &T) -> Result<(), Self::Error>;
}
