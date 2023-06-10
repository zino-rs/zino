use crate::Group;
use serde::{Deserialize, Serialize};
use zino_core::{
    datetime::DateTime,
    extension::JsonObjectExt,
    model::{Model, ModelHooks},
    request::Validation,
    Map, Uuid,
};
use zino_derive::{ModelAccessor, Schema};

#[cfg(feature = "tags")]
use crate::Tag;

#[cfg(any(feature = "owner-id", feature = "maintainer-id"))]
use crate::User;

/// The `policy` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Policy {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[cfg(feature = "namespace")]
    #[schema(default_value = "Policy::model_namespace", index_type = "hash")]
    namespace: String,
    #[cfg(feature = "visibility")]
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(reference = "Group")]
    tenant_id: Uuid, // group.id, group.namespace = "*:policy"
    #[schema(not_null)]
    resource: String,
    actions: Vec<String>,
    effect: String,
    valid_from: DateTime,
    expires_at: DateTime,
    #[cfg(feature = "tags")]
    #[schema(reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:policy"

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

impl Model for Policy {
    #[inline]
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
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
        #[cfg(feature = "tags")]
        if let Some(tags) = data.parse_array("tags") {
            self.tags = tags;
        }
        validation
    }
}

impl ModelHooks for Policy {}

impl Policy {
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
