//! The `user` model and related services.

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use zino_core::{
    auth::{AccessKeyId, UserSession},
    bail,
    datetime::DateTime,
    error::Error,
    extension::JsonObjectExt,
    model::{Model, ModelHooks},
    orm::ModelHelper,
    validation::Validation,
    Map, Uuid,
};
use zino_derive::{DecodeRow, ModelAccessor, Schema};

#[cfg(feature = "tags")]
use crate::tag::Tag;

mod jwt_auth;
mod status;

pub use jwt_auth::JwtAuthService;
pub use status::UserStatus;

#[cfg(feature = "visibility")]
mod visibility;

#[cfg(feature = "visibility")]
pub use visibility::UserVisibility;

/// The `user` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, DecodeRow, Schema, ModelAccessor)]
#[serde(default)]
pub struct User {
    // Basic fields.
    #[schema(read_only)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[cfg(feature = "namespace")]
    #[schema(default_value = "User::model_namespace", index_type = "hash")]
    namespace: String,
    #[cfg(feature = "visibility")]
    #[schema(type_name = "String", default_value = "UserVisibility::default")]
    visibility: UserVisibility,
    #[schema(
        type_name = "String",
        default_value = "UserStatus::default",
        index_type = "hash"
    )]
    status: UserStatus,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(unique)]
    union_id: String,
    #[schema(not_null, unique, write_only)]
    access_key_id: String,
    #[schema(not_null, unique, write_only)]
    account: String,
    #[schema(not_null, write_only)]
    password: String,
    nickname: String,
    #[schema(format = "uri")]
    avatar: String,
    #[schema(format = "uri")]
    website: String,
    #[schema(format = "email")]
    email: String,
    location: String,
    locale: String,
    #[schema(format = "phone_number")]
    mobile: String,
    #[schema(snapshot, nonempty, unique_items, index_type = "gin")]
    roles: Vec<String>,
    #[cfg(feature = "tags")]
    #[schema(unique_items, reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:user"

    // Security.
    last_login_at: DateTime,
    last_login_ip: String,
    current_login_at: DateTime,
    current_login_ip: String,
    login_count: u32,
    failed_login_count: u8,

    // Extensions.
    content: Map,
    extra: Map,

    // Revisions.
    #[cfg(feature = "owner-id")]
    #[schema(reference = "User")]
    owner_id: Option<Uuid>, // user.id
    #[cfg(feature = "maintainer-id")]
    #[schema(reference = "User")]
    maintainer_id: Option<Uuid>, // user.id
    #[schema(read_only, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
    #[cfg(feature = "edition")]
    edition: u32,
}

impl Model for User {
    #[inline]
    fn new() -> Self {
        Self {
            id: Uuid::now_v7(),
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
        if let Some(union_id) = data.parse_string("union_id") {
            self.union_id = union_id.into_owned();
        }
        if let Some(account) = data.parse_string("account") {
            self.account = account.into_owned();
        }
        if let Some(password) = data.parse_string("password") {
            match User::encrypt_password(&password) {
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
        #[cfg(feature = "tags")]
        if let Some(tags) = data.parse_array("tags") {
            self.tags = tags;
        }
        #[cfg(feature = "owner-id")]
        if let Some(result) = data.parse_uuid("owner_id") {
            match result {
                Ok(owner_id) => self.owner_id = Some(owner_id),
                Err(err) => validation.record_fail("owner_id", err),
            }
        }
        #[cfg(feature = "maintainer-id")]
        if let Some(result) = data.parse_uuid("maintainer_id") {
            match result {
                Ok(maintainer_id) => self.maintainer_id = Some(maintainer_id),
                Err(err) => validation.record_fail("maintainer_id", err),
            }
        }
        validation
    }
}

impl ModelHooks for User {
    #[cfg(feature = "maintainer-id")]
    type Extension = UserSession<Uuid, String>;

    #[cfg(feature = "maintainer-id")]
    #[inline]
    async fn after_extract(&mut self, session: Self::Extension) -> Result<(), Error> {
        self.maintainer_id = Some(*session.user_id());
        Ok(())
    }

    #[cfg(feature = "maintainer-id")]
    #[inline]
    async fn before_validation(
        data: &mut Map,
        extension: Option<&Self::Extension>,
    ) -> Result<(), Error> {
        if let Some(session) = extension {
            data.upsert("maintainer_id", session.user_id().to_string());
        }
        Ok(())
    }
}

impl User {
    /// Sets the `access_key_id`.
    #[inline]
    pub fn set_access_key_id(&mut self, access_key_id: AccessKeyId) {
        self.access_key_id = access_key_id.to_string();
    }

    /// Sets the `roles` field.
    pub fn set_roles(&mut self, roles: Vec<&str>) -> Result<(), Error> {
        let num_roles = roles.len();
        let special_roles = ["superuser", "user", "guest"];
        for role in &roles {
            if special_roles.contains(role) && num_roles != 1 {
                bail!("the special role `{}` is exclusive", role);
            } else if !USER_ROLE_PATTERN.is_match(role) {
                bail!("the role `{}` is invalid", role);
            } else if role.is_empty() {
                bail!("the `roles` can not contain empty values");
            }
        }
        self.roles = roles.into_iter().map(|s| s.to_owned()).collect();
        Ok(())
    }

    /// Returns the `union_id` field.
    #[inline]
    pub fn union_id(&self) -> &str {
        &self.union_id
    }

    /// Returns the `access_key_id` field.
    #[inline]
    pub fn access_key_id(&self) -> &str {
        self.access_key_id.as_str()
    }

    /// Returns the `roles` field.
    #[inline]
    pub fn roles(&self) -> &[String] {
        self.roles.as_slice()
    }

    /// Returns a session for the user.
    pub fn user_session(&self) -> UserSession<Uuid, String> {
        let mut user_session = UserSession::new(self.id, None);
        user_session.set_access_key_id(self.access_key_id().into());
        user_session.set_roles(self.roles());
        user_session
    }

    /// Returns the user info as standard claims defined in the
    /// [OpenID Connect](https://openid.net/specs/openid-connect-core-1_0.html#StandardClaims).
    pub fn user_info(&self) -> Map {
        let mut claims = self.standard_claims();
        claims.upsert("sub", self.id.to_string());
        claims.upsert("updated_at", self.updated_at.timestamp());
        if !claims.get_str("name").is_some_and(|s| !s.is_empty()) {
            claims.upsert("name", self.name.as_str());
        }
        if !claims.get_str("nickname").is_some_and(|s| !s.is_empty()) {
            claims.upsert("nickname", self.nickname.as_str());
        }
        if !claims.get_str("picture").is_some_and(|s| !s.is_empty()) {
            claims.upsert("picture", self.avatar.as_str());
        }
        if !claims.get_str("website").is_some_and(|s| !s.is_empty()) {
            claims.upsert("website", self.website.as_str());
        }
        if !claims.get_str("email").is_some_and(|s| !s.is_empty()) {
            claims.upsert("email", self.email.as_str());
        }
        if !claims.get_object("address").is_some_and(|o| !o.is_empty()) {
            claims.upsert(
                "address",
                Map::from_entry("locality", self.location.as_str()),
            );
        }
        if !claims.get_str("locale").is_some_and(|s| !s.is_empty()) {
            claims.upsert("locale", self.locale.as_str());
        }
        if !claims
            .get_str("phone_number")
            .is_some_and(|s| !s.is_empty())
        {
            claims.upsert("phone_number", self.mobile.as_str());
        }
        claims
    }
}

/// Regex for the user role.
static USER_ROLE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z]+[a-z:]+[a-z]+$").expect("fail to create a regex for the user role")
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

        let user_session = alice.user_session();
        assert!(user_session.is_admin());
        assert!(!user_session.is_worker());
        assert!(user_session.is_auditor());
        assert!(user_session.has_role("admin:user"));
        assert!(!user_session.has_role("admin:group"));
        assert!(user_session.has_role("auditor:log"));
        assert!(!user_session.has_role("auditor_record"));
    }
}
