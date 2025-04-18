//! The `user` model and related services.

use serde::{Deserialize, Serialize};
use zino_auth::{AccessKeyId, UserSession};
use zino_core::{
    Map, Uuid, bail,
    datetime::DateTime,
    error::Error,
    extension::JsonObjectExt,
    model::{Model, ModelHooks},
    validation::Validation,
};
use zino_derive::{DecodeRow, Entity, ModelAccessor, Schema};
use zino_orm::ModelHelper;

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
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, DecodeRow, Entity, Schema, ModelAccessor,
)]
#[serde(default)]
#[schema(auto_rename)]
pub struct User {
    // Basic fields.
    #[schema(read_only)]
    id: Uuid,
    #[schema(not_null)]
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
    mobile: String,
    #[schema(snapshot, nonempty, unique_items, index_type = "gin")]
    roles: Vec<String>,
    #[cfg(feature = "tags")]
    #[schema(unique_items, reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:user"

    // Security.
    last_login_at: DateTime,
    #[schema(format = "ip")]
    last_login_ip: String,
    current_login_at: DateTime,
    #[schema(format = "ip")]
    current_login_ip: String,
    login_count: u32,
    failed_login_count: u8,

    // Extensions.
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
    const MODEL_NAME: &'static str = "user";

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
        if let Some(result) = data.parse_array("tags") {
            match result {
                Ok(tags) => self.tags = tags,
                Err(err) => validation.record_fail("tags", err),
            }
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
    type Data = ();
    #[cfg(feature = "maintainer-id")]
    type Extension = UserSession<Uuid, String>;
    #[cfg(not(feature = "maintainer-id"))]
    type Extension = ();

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
                bail!("special role `{}` is exclusive", role);
            } else if role.is_empty() {
                bail!("`roles` can not contain empty values");
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
}

#[cfg(test)]
mod tests {
    use super::User;
    use zino_core::{Map, extension::JsonObjectExt, model::Model};

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
