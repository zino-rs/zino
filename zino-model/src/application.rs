use crate::User;
use serde::{Deserialize, Serialize};
use zino_core::{
    authentication::AccessKeyId,
    datetime::DateTime,
    extension::JsonObjectExt,
    model::{Model, ModelHooks},
    request::Validation,
    Map, Uuid,
};
use zino_derive::{ModelAccessor, Schema};

#[cfg(feature = "tags")]
use crate::Tag;

/// The `application` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Application {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[cfg(feature = "namespace")]
    #[schema(default_value = "Application::model_namespace", index_type = "hash")]
    namespace: String,
    #[cfg(feature = "visibility")]
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(reference = "User")]
    manager_id: Uuid, // user.id
    #[schema(not_null, unique, writeonly)]
    access_key_id: String,
    #[cfg(feature = "tags")]
    #[schema(reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:application"

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
    #[schema(readonly, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
    #[cfg(feature = "edition")]
    edition: u32,
}

impl Model for Application {
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
        if let Some(description) = data.parse_string("description") {
            self.description = description.into_owned();
        }
        if let Some(result) = data.parse_uuid("manager_id") {
            match result {
                Ok(manager_id) => self.manager_id = manager_id,
                Err(err) => validation.record_fail("manager_id", err),
            }
        }
        #[cfg(feature = "tags")]
        if let Some(tags) = data.parse_array("tags") {
            self.tags = tags;
        }
        validation
    }
}

impl ModelHooks for Application {}

impl Application {
    /// Sets the `access_key_id`.
    #[inline]
    pub fn set_access_key_id(&mut self, access_key_id: AccessKeyId) {
        self.access_key_id = access_key_id.to_string();
    }

    /// Sets the `owner_id` field.
    #[cfg(feature = "owner-id")]
    #[inline]
    pub fn set_owner_id(&mut self, owner_id: Uuid) {
        self.owner_id = Some(owner_id);
    }

    /// Sets the `maintainer_id` field.
    #[cfg(feature = "maintainer-id")]
    #[inline]
    pub fn set_maintainer_id(&mut self, maintainer_id: Uuid) {
        self.maintainer_id = Some(maintainer_id);
    }
}