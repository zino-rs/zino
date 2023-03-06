use regex::Regex;
use serde::{Deserialize, Serialize};
use zino_core::{
    authentication::AccessKeyId, datetime::DateTime, error::Error, model::Model,
    request::Validation, Map, Uuid,
};
use zino_derive::Schema;

/// The user model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct User {
    // Basic fields.
    id: Uuid,
    #[schema(not_null, index = "text")]
    name: String,
    #[schema(default = "User::model_namespace", index = "hash")]
    namespace: String,
    #[schema(default = "internal")]
    visibility: String,
    #[schema(default = "active", index = "hash")]
    status: String,
    #[schema(index = "text")]
    description: String,

    // Info fields.
    #[schema(not_null)]
    access_key_id: String,
    #[schema(not_null)]
    account: String,
    #[schema(not_null)]
    password: String,
    mobile: String,
    email: String,
    avatar: String,
    roles: Vec<String>,
    #[schema(index = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:user"

    // Extensions.
    content: Map,
    metrics: Map,
    extras: Map,

    // Revisions.
    manager_id: Uuid,    // user.id
    maintainer_id: Uuid, // user.id
    #[schema(default = "now", index = "btree")]
    created_at: DateTime,
    #[schema(default = "now", index = "btree")]
    updated_at: DateTime,
    version: u64,
    edition: u32,
}

impl Model for User {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            access_key_id: AccessKeyId::new().to_string(),
            ..Self::default()
        }
    }

    fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        if let Some(result) = Validation::parse_uuid(data.get("id")) {
            match result {
                Ok(id) => self.id = id,
                Err(err) => validation.record_fail("id", err),
            }
        }
        if let Some(name) = Validation::parse_string(data.get("name")) {
            self.name = name;
        }
        if self.name.is_empty() {
            validation.record("name", "should be nonempty");
        }
        if let Some(roles) = Validation::parse_array(data.get("roles")) {
            if let Err(err) = self.set_roles(roles) {
                validation.record_fail("roles", err);
            }
        }
        if self.roles.is_empty() && !validation.contains_key("roles") {
            validation.record("roles", "should be nonempty");
        }
        validation
    }
}

super::impl_model_accessor!(
    User,
    id,
    name,
    namespace,
    visibility,
    status,
    description,
    content,
    metrics,
    extras,
    manager_id,
    maintainer_id,
    created_at,
    updated_at,
    version,
    edition
);

impl User {
    /// Sets the `roles` of the user.
    pub fn set_roles(&mut self, roles: Vec<String>) -> Result<(), Error> {
        let num_roles = roles.len();
        let special_roles = ["superuser", "guest"];
        for role in &roles {
            let role = role.as_str();
            if special_roles.contains(&role) && num_roles != 1 {
                let message = format!("the special role `{role}` is exclusive");
                return Err(Error::new(message));
            } else if !USER_ROLE_PATTERN.is_match(role) {
                let message = format!("the role `{role}` is invalid");
                return Err(Error::new(message));
            }
        }
        self.roles = roles;
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
    Regex::new(r"^[a-z]+[a-z:]+[a-z]+$").expect("failed to create the user role pattern")
});

#[cfg(test)]
mod tests {
    use super::User;
    use zino_core::{extend::JsonObjectExt, model::Model, Map};

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
