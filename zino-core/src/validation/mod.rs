//! Generic validator and common validation rules.
use crate::{error::Error, extension::JsonObjectExt, Map, SharedString};

mod validator;

pub use validator::{
    DateTimeValidator, DateValidator, EmailValidator, HostValidator, HostnameValidator,
    IpAddrValidator, Ipv4AddrValidator, Ipv6AddrValidator, TimeValidator, UriValidator,
    UuidValidator, Validator,
};

/// A record of validation results.
#[derive(Debug, Default)]
pub struct Validation {
    failed_entries: Vec<(SharedString, Error)>,
}

impl Validation {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            failed_entries: Vec::new(),
        }
    }

    /// Creates a new instance with the entry.
    #[inline]
    pub fn from_entry(key: impl Into<SharedString>, err: impl Into<Error>) -> Self {
        let failed_entries = vec![(key.into(), err.into())];
        Self { failed_entries }
    }

    /// Records an entry with the supplied message.
    #[inline]
    pub fn record(&mut self, key: impl Into<SharedString>, message: impl Into<SharedString>) {
        self.failed_entries.push((key.into(), Error::new(message)));
    }

    /// Records an entry for the error.
    #[inline]
    pub fn record_fail(&mut self, key: impl Into<SharedString>, err: impl Into<Error>) {
        self.failed_entries.push((key.into(), err.into()));
    }

    /// Validates the string value with a specific format.
    pub fn validate_format(&mut self, key: impl Into<SharedString>, value: &str, format: &str) {
        match format {
            "date" => {
                if let Err(err) = DateValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "date-time" => {
                if let Err(err) = DateTimeValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "email" => {
                if let Err(err) = EmailValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "host" => {
                if let Err(err) = HostValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "hostname" => {
                if let Err(err) = HostnameValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ip" => {
                if let Err(err) = IpAddrValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ipv4" => {
                if let Err(err) = Ipv4AddrValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ipv6" => {
                if let Err(err) = Ipv6AddrValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "time" => {
                if let Err(err) = TimeValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "uri" => {
                if let Err(err) = UriValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "uuid" => {
                if let Err(err) = UuidValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            _ => {
                tracing::warn!("supported format `{format}`");
            }
        }
    }

    /// Returns true if the validation contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.failed_entries.iter().any(|(field, _)| field == key)
    }

    /// Returns `true` if the validation is success.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.failed_entries.is_empty()
    }

    /// Consumes the validation and returns as a json object.
    #[must_use]
    pub fn into_map(self) -> Map {
        let failed_entries = self.failed_entries;
        let mut map = Map::with_capacity(failed_entries.len());
        for (key, err) in failed_entries {
            map.upsert(key, err.to_string());
        }
        map
    }
}
