use crate::Tag;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use zino_core::{
    authentication::AccessKeyId, database::ModelHooks, datetime::DateTime, error::Error,
    extension::JsonObjectExt, model::Model, request::Validation, Map, Uuid,
};
use zino_derive::{ModelAccessor, Schema};

/// The user model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct User {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "User::model_namespace", index_type = "hash")]
    namespace: String,
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null, unique, writeonly)]
    access_key_id: String,
    #[schema(not_null, unique, writeonly)]
    account: String,
    #[schema(not_null, writeonly)]
    password: String,
    mobile: String,
    email: String,
    avatar: String,
    #[schema(index_type = "gin")]
    roles: Vec<String>,
    #[schema(reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:user"

    // Extensions.
    content: Map,
    extra: Map,

    // Revisions.
    #[schema(reference = "User")]
    owner_id: Option<Uuid>, // user.id
    #[schema(reference = "User")]
    maintainer_id: Option<Uuid>, // user.id
    #[schema(readonly, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
    edition: u32,
}

impl Model for User {
    #[inline]
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            access_key_id: AccessKeyId::new().to_string(),
            ..Self::default()
        }
    }

    fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        if let Some(result) = data.parse_uuid("id") {
            match result {
                Ok(id) => self.id = id,
                Err(err) => validation.record_fail("id", err),
            }
        }
        if let Some(name) = data.parse_string("name") {
            self.name = name.into_owned();
        }
        if let Some(account) = data.parse_string("account") {
            self.account = account.into_owned();
        }
        if let Some(password) = data.parse_string("password") {
            match User::encrypt_password(password.as_bytes()) {
                Ok(password) => self.password = password,
                Err(err) => validation.record_fail("password", err),
            }
        }
        if let Some(roles) = data.parse_str_array("roles") {
            if let Err(err) = self.set_roles(roles) {
                validation.record_fail("roles", err);
            }
        }
        if self.roles.is_empty() && !validation.contains_key("roles") {
            validation.record("roles", "should be nonempty");
        }
        if let Some(tags) = data.parse_array("tags") {
            self.tags = tags;
        }
        validation
    }
}

impl ModelHooks for User {}

impl User {
    /// Sets the `access_key_id`.
    #[inline]
    pub fn set_access_key_id(&mut self, access_key_id: AccessKeyId) {
        self.access_key_id = access_key_id.to_string();
    }

    /// Returns a reference to the `access_key_id`.
    #[inline]
    pub fn access_key_id(&self) -> &[u8] {
        self.access_key_id.as_ref()
    }

    /// Sets the `roles` of the user.
    pub fn set_roles(&mut self, roles: Vec<&str>) -> Result<(), Error> {
        let num_roles = roles.len();
        let special_roles = ["superuser", "user", "guest"];
        for role in &roles {
            if special_roles.contains(role) && num_roles != 1 {
                let message = format!("the special role `{role}` is exclusive");
                return Err(Error::new(message));
            } else if !USER_ROLE_PATTERN.is_match(role) {
                let message = format!("the role `{role}` is invalid");
                return Err(Error::new(message));
            }
        }
        self.roles = roles.into_iter().map(|s| s.to_owned()).collect();
        Ok(())
    }

    /// Returns the `roles` field.
    #[inline]
    pub fn roles(&self) -> &[String] {
        self.roles.as_slice()
    }

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

/// User role pattern.
static USER_ROLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z]+[a-z:]+[a-z]+$").expect("fail to create the user role pattern")
});

#[cfg(test)]
mod tests {
    use super::User;
    use zino_core::{extension::JsonObjectExt, model::Model, Map};

    #[test]
    fn it_checks_user_roles() {
        let mut alice = User::new();
        let mut data = Map::new();
        data.upsert("name", "alice");
        data.upsert("roles", vec!["admin:user", "auditor"]);

        let validation = alice.read_map(&data);
        assert!(validation.is_success());
        assert!(alice.is_admin());
        assert!(!alice.is_worker());
        assert!(alice.is_auditor());
        assert!(alice.has_role("admin:user"));
        assert!(!alice.has_role("admin:group"));
        assert!(alice.has_role("auditor:log"));
        assert!(!alice.has_role("auditor_record"));
    }
}
