use super::{AccessKeyId, JwtClaims, SessionId};
use crate::{application::APP_DOMAIN, error::Error, extension::JsonObjectExt};
use sha2::Sha256;
use std::str::FromStr;

/// Role-based user sessions.
#[derive(Debug, Clone)]
pub struct UserSession<U, R = String> {
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
    pub fn set_roles(&mut self, roles: impl Into<Vec<R>>) {
        self.roles = roles.into();
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

impl<U, R> UserSession<U, R>
where
    U: FromStr,
    R: FromStr,
    <U as FromStr>::Err: std::error::Error,
{
    /// Attempts to construct an instance from a `JwtClaims`.
    pub fn try_from_jwt_claims(claims: JwtClaims) -> Result<Self, Error> {
        let user_id = claims
            .subject()
            .ok_or_else(|| Error::new("the subject of a JWT token shoud be specified"))?
            .parse()?;
        let mut user_session = Self::new(user_id, None);
        if let Some(roles) = claims.data().parse_array("roles") {
            user_session.set_roles(roles);
        }
        Ok(user_session)
    }
}

impl<U> UserSession<U, String> {
    /// Returns `true` if the user has a role of `superuser`.
    #[inline]
    pub fn is_superuser(&self) -> bool {
        self.roles() == ["superuser"]
    }

    /// Returns `true` if the user has a role of `user`.
    #[inline]
    pub fn is_user(&self) -> bool {
        self.roles() == ["user"]
    }

    /// Returns `true` if the user has a role of `guest`.
    #[inline]
    pub fn is_guest(&self) -> bool {
        self.roles() == ["guest"]
    }

    /// Returns `true` if the user has a role of `admin`.
    pub fn is_admin(&self) -> bool {
        let role = "admin";
        let role_prefix = format!("{role}:");
        for r in &self.roles {
            if r == role || r.starts_with(&role_prefix) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the user has a role of `worker`.
    pub fn is_worker(&self) -> bool {
        let role = "worker";
        let role_prefix = format!("{role}:");
        for r in &self.roles {
            if r == role || r.starts_with(&role_prefix) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the user has a role of `auditor`.
    pub fn is_auditor(&self) -> bool {
        let role = "auditor";
        let role_prefix = format!("{role}:");
        for r in &self.roles {
            if r == role || r.starts_with(&role_prefix) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the user has one of the roles: `superuser`, `user`,
    /// `admin`, `worker` and `auditor`.
    pub fn has_user_role(&self) -> bool {
        self.is_superuser()
            || self.is_user()
            || self.is_admin()
            || self.is_worker()
            || self.is_auditor()
    }

    /// Returns `true` if the user has a role of `superuser` or `admin`.
    pub fn has_admin_role(&self) -> bool {
        self.is_superuser() || self.is_admin()
    }

    /// Returns `true` if the user has a role of `superuser` or `worker`.
    pub fn has_worker_role(&self) -> bool {
        self.is_superuser() || self.is_worker()
    }

    /// Returns `true` if the user has a role of `superuser` or `auditor`.
    pub fn has_auditor_role(&self) -> bool {
        self.is_superuser() || self.is_auditor()
    }

    /// Returns `true` if the user has the specific `role`.
    pub fn has_role(&self, role: &str) -> bool {
        let length = role.len();
        for r in &self.roles {
            if r == role {
                return true;
            } else {
                let remainder = if r.len() > length {
                    r.strip_prefix(role)
                } else {
                    role.strip_prefix(r.as_str())
                };
                if let Some(s) = remainder && s.starts_with(':') {
                    return true;
                }
            }
        }
        false
    }
}
