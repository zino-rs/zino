use super::{AccessKeyId, SessionId};
use crate::application::APP_DOMAIN;
use sha2::Sha256;

/// Role-based user sessions.
#[derive(Debug, Clone)]
pub struct UserSession<U, R> {
    /// User ID.
    user_id: U,
    /// Session ID.
    session_id: Option<SessionId>,
    /// Access key ID.
    access_key_id: Option<AccessKeyId>,
    /// A list of user roles.
    roles: Vec<R>,
}

impl<U, R> UserSession<U, R> {
    /// Creates a new instance with empty roles.
    #[inline]
    pub fn new(user_id: U, session_id: impl Into<Option<SessionId>>) -> Self {
        Self {
            user_id,
            session_id: session_id.into(),
            access_key_id: None,
            roles: Vec::new(),
        }
    }

    /// Sets the session ID.
    #[inline]
    pub fn set_session_id(&mut self, session_id: SessionId) {
        self.session_id = Some(session_id);
    }

    /// Sets the access key ID.
    #[inline]
    pub fn set_access_key_id(&mut self, access_key_id: AccessKeyId) {
        if self.session_id.is_none() {
            let session_id = SessionId::new::<Sha256>(*APP_DOMAIN, access_key_id.as_ref());
            self.session_id = Some(session_id);
        }
        self.access_key_id = Some(access_key_id);
    }

    /// Sets the user roles.
    #[inline]
    pub fn set_roles(&mut self, roles: Vec<R>) {
        self.roles = roles;
    }

    /// Returns the user ID.
    #[inline]
    pub fn user_id(&self) -> &U {
        &self.user_id
    }

    /// Returns the session ID.
    #[inline]
    pub fn session_id(&self) -> Option<&SessionId> {
        self.session_id.as_ref()
    }

    /// Returns the access key ID.
    #[inline]
    pub fn access_key_id(&self) -> Option<&AccessKeyId> {
        self.access_key_id.as_ref()
    }

    /// Returns the roles.
    #[inline]
    pub fn roles(&self) -> &[R] {
        &self.roles
    }
}
