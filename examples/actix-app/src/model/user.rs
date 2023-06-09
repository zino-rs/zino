use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{ModelAccessor, Schema};
use zino_model::Tag;

/// The `User` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct User {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null, writeonly)]
    access_key_id: String,
    #[schema(not_null, writeonly)]
    account: String,
    #[schema(not_null, writeonly)]
    password: String,
    mobile: String,
    email: String,
    avatar: String,
    roles: Vec<String>,
    #[schema(reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:user"

    // Extensions.
    content: Map,
    extra: Map,

    // Revisions.
    #[schema(readonly, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
}

impl User {
    /// Sets the `roles` of the user.
    pub fn set_roles(&mut self, roles: Vec<&str>) -> Result<(), Error> {
        self.roles = roles.into_iter().map(|s| s.to_owned()).collect();
        Ok(())
    }
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
