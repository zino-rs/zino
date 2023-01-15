use crate::SharedString;
use std::fmt;

/// Session Identification URI.
/// See [the spec](https://www.w3.org/TR/WD-session-id).
pub struct SessionId {
    /// Specifies the realm within which linkage of the identifier is possible.
    /// Realms have the same format as DNS names.
    realm: SharedString,
    /// Unstructured random integer specific to realm generated using a procedure with
    /// a negligible probability of collision. The identifier is encoded using base64.
    identifier: SharedString,
    /// Optional extension of identifier field used to differentiate concurrent uses of
    /// the same session identifier. The thread field is an integer encoded in hexadecimal.
    thread: Option<u16>,
    /// Optional Hexadecimal encoded Integer containing a monotonically increasing counter value.
    /// A client should increment the count field after each operation.
    count: Option<u16>,
}

impl SessionId {
    /// Creates a new instance.
    #[inline]
    pub fn new(realm: impl Into<SharedString>, identifier: impl Into<SharedString>) -> Self {
        Self {
            realm: realm.into(),
            identifier: identifier.into(),
            thread: None,
            count: None,
        }
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let realm = &self.realm;
        let identifier = &self.identifier;
        match self.thread {
            Some(thread) => match self.count {
                Some(count) => write!(f, "SID:ANON:{realm}:{identifier}-{thread:x}:{count:x}"),
                None => write!(f, "SID:ANON:{realm}:{identifier}-{thread:x}"),
            },
            None => match self.count {
                Some(count) => write!(f, "SID:ANON:{realm}:{identifier}:{count:x}"),
                None => write!(f, "SID:ANON:{realm}:{identifier}"),
            },
        }
    }
}
