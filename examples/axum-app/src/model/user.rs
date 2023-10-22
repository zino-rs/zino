use super::Tag;
use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{ModelAccessor, ModelHooks, Schema};
use zino_model::user::JwtAuthService;

/// The `User` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor, ModelHooks)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct User {
    // Basic fields.
    #[schema(auto_increment, readonly)]
    id: i64,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Inactive", index_type = "hash")]
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
    #[schema(snapshot)]
    roles: Vec<String>,
    #[schema(reference = "Tag")]
    tags: Vec<i64>, // tag.id, tag.namespace = "*:user"

    // Security.
    last_login_at: DateTime,
    last_login_ip: String,
    current_login_at: DateTime,
    current_login_ip: String,
    login_count: u32,

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
            access_key_id: AccessKeyId::new().to_string(),
            ..Self::default()
        }
    }

    fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        if let Some(result) = data.parse_i64("id") {
            match result {
                Ok(id) => self.id = id,
                Err(err) => validation.record_fail("id", err),
            }
        }
        if let Some(name) = data.parse_string("name") {
            self.name = name.into_owned();
        }
        if let Some(description) = data.parse_string("description") {
            self.description = description.into_owned();
        }
        if let Some(account) = data.parse_string("account") {
            self.account = account.into_owned();
        }
        if let Some(password) = data.parse_string("password") {
            match Self::encrypt_password(&password) {
                Ok(password) => self.password = password,
                Err(err) => validation.record_fail("password", err),
            }
        }
        if let Some(mobile) = data.parse_string("mobile") {
            self.mobile = mobile.into_owned();
        }
        if let Some(email) = data.parse_string("email") {
            self.email = email.into_owned();
        }
        if let Some(avatar) = data.parse_string("avatar") {
            self.avatar = avatar.into_owned();
        }
        if let Some(roles) = data.parse_str_array("roles") {
            if let Err(err) = self.set_roles(roles) {
                validation.record_fail("roles", err);
            }
        }
        if self.roles.is_empty() && !validation.contains_key("roles") {
            validation.record("roles", "should be nonempty");
        }
        if let Some(tags) = data.get_i64_array("tags") {
            self.tags = tags;
        }
        validation
    }
}

impl JwtAuthService<i64> for User {
    const LOGIN_AT_FIELD: Option<&'static str> = Some("current_login_at");
    const LOGIN_IP_FIELD: Option<&'static str> = Some("current_login_ip");
}
