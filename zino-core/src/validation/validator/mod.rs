//! Common validation rules.

mod alphabetic;
mod alphanumeric;
mod ascii;
mod ascii_alphabetic;
mod ascii_alphanumeric;
mod ascii_digit;
mod ascii_hexdigit;
mod ascii_lowercase;
mod ascii_uppercase;
mod date;
mod date_time;
mod email;
mod host;
mod hostname;
mod ip_addr;
mod ipv4_addr;
mod ipv6_addr;
mod lowercase;
mod numeric;
mod regex;
mod time;
mod uppercase;
mod uri;
mod uuid;

pub use alphabetic::AlphabeticValidator;
pub use alphanumeric::AlphanumericValidator;
pub use ascii::AsciiValidator;
pub use ascii_alphabetic::AsciiAlphabeticValidator;
pub use ascii_alphanumeric::AsciiAlphanumericValidator;
pub use ascii_digit::AsciiDigitValidator;
pub use ascii_hexdigit::AsciiHexdigitValidator;
pub use ascii_lowercase::AsciiLowercaseValidator;
pub use ascii_uppercase::AsciiUppercaseValidator;
pub use date::DateValidator;
pub use date_time::DateTimeValidator;
pub use email::EmailValidator;
pub use host::HostValidator;
pub use hostname::HostnameValidator;
pub use ip_addr::IpAddrValidator;
pub use ipv4_addr::Ipv4AddrValidator;
pub use ipv6_addr::Ipv6AddrValidator;
pub use lowercase::LowercaseValidator;
pub use numeric::NumericValidator;
pub use regex::RegexValidator;
pub use time::TimeValidator;
pub use uppercase::UppercaseValidator;
pub use uri::UriValidator;
pub use uuid::UuidValidator;

/// A generic validator.
pub trait Validator<T: ?Sized> {
    /// The error type.
    type Error: Into<crate::error::Error>;

    /// Validates the data.
    fn validate(&self, data: &T) -> Result<(), Self::Error>;
}
