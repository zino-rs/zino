use super::{AccessKeyId, JwtClaims, SessionId};
use crate::{application::APP_DOMAIN, crypto::Hash, error::Error, extension::JsonObjectExt};
use std::str::FromStr;

/// Role-based user sessions.
#[derive(Debug, Clone)]
pub struct UserSession<U, R = String, T = U> {
    /// User ID.
    user_id: U,
    /// Session ID.
    session_id: Option<SessionId>,
    /// Access key ID.
    access_key_id: Option<AccessKeyId>,
    /// A list of user roles.
    roles: Vec<R>,
    /// Tenant ID.
    tenant_id: Option<T>,
}

impl<U, R, T> UserSession<U, R, T> {
    /// Creates a new instance with empty roles.
    #[inline]
    pub fn new(user_id: U, session_id: impl Into<Option<SessionId>>) -> Self {
        Self {
            user_id,
            session_id: session_id.into(),
            access_key_id: None,
            roles: Vec::new(),
            tenant_id: None,
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
            let session_id = SessionId::new::<Hash>(*APP_DOMAIN, access_key_id.as_ref());
            self.session_id = Some(session_id);
        }
        self.access_key_id = Some(access_key_id);
    }

    /// Sets the user roles.
    #[inline]
    pub fn set_roles(&mut self, roles: impl Into<Vec<R>>) {
        self.roles = roles.into();
    }

    /// Sets the tenant ID.
    #[inline]
    pub fn set_tenant_id(&mut self, tenant_id: T) {
        self.tenant_id = Some(tenant_id);
    }

    /// Returns the user ID.
    #[inline]
    pub fn user_id(&self) -> &U {
        &self.user_id
    }

    /// Returns the tenant ID.
    #[inline]
    pub fn tenant_id(&self) -> Option<&T> {
        self.tenant_id.as_ref()
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

impl<U, R, T> UserSession<U, R, T>
where
    U: FromStr,
    R: FromStr,
    T: FromStr,
    <U as FromStr>::Err: std::error::Error,
{
    /// Attempts to construct an instance from a `JwtClaims`.
    pub fn try_from_jwt_claims(claims: JwtClaims) -> Result<Self, Error> {
        let data = claims.data();
        let user_id = claims
            .subject()
            .map(|s| s.into())
            .or_else(|| data.parse_string("uid"))
            .ok_or_else(|| Error::new("the subject of a JWT token shoud be specified"))?
            .parse()?;
        let mut user_session = Self::new(user_id, None);
        if let Some(roles) = data
            .parse_array("roles")
            .or_else(|| data.parse_array("role"))
        {
            user_session.set_roles(roles);
        }
        if let Some(tenant_id) = data
            .parse_string("tenant_id")
            .or_else(|| data.parse_string("tid"))
            .and_then(|s| s.parse().ok())
        {
            user_session.set_tenant_id(tenant_id);
        }
        Ok(user_session)
    }
}

impl<U, T> UserSession<U, String, T> {
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

    /// Returns `true` if the user has any of the specific `roles`.
    pub fn has_any_roles(&self, roles: &[&str]) -> bool {
        for role in roles {
            if self.has_role(role) {
                return true;
            }
        }
        false
    }

    /// Returns `true` if the user has all of the specific `roles`.
    pub fn has_all_roles(&self, roles: &[&str]) -> bool {
        for role in roles {
            if !self.has_role(role) {
                return false;
            }
        }
        true
    }
}
