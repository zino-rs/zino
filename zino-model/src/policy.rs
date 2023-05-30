use crate::{Group, Tag, User};
use serde::{Deserialize, Serialize};
use zino_core::{
    datetime::DateTime, extension::JsonObjectExt, model::Model, request::Validation, Map, Uuid,
};
use zino_derive::{ModelAccessor, Schema};

/// The policy model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Policy {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Policy::model_namespace", index_type = "hash")]
    namespace: String,
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(reference = "Group")]
    tenant_id: Uuid, // group.id, group.namespace = "*:policy", group.subject = "user"
    #[schema(not_null)]
    resource: String,
    actions: Vec<String>,
    effect: String,
    valid_from: DateTime,
    expires_at: DateTime,
    #[schema(reference = "Tag", index_type = "gin")]
    tags: Vec<Uuid>, // tag.id, tag.namespace = "*:policy"

    // Extensions.
    content: Map,
    extra: Map,

    // Revisions.
    #[schema(reference = "User")]
    owner_id: Uuid, // user.id
    #[schema(reference = "User")]
    maintainer_id: Uuid, // user.id
    #[schema(readonly, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
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
        if self.name.is_empty() {
            validation.record("name", "should be nonempty");
        }
        validation
    }
}
