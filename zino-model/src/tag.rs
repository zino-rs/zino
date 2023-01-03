use serde::{Deserialize, Serialize};
use zino_core::{DateTime, Map, Model, Schema, Uuid, Validation};
use zino_derive::Schema;

/// The tag model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Tag {
    // Basic fields.
    id: Uuid,
    #[schema(not_null, index = "text")]
    name: String,
    #[schema(default = "Tag::model_namespace", index = "hash")]
    namespace: String,
    #[schema(default = "internal")]
    visibility: String,
    #[schema(default = "active", index = "hash")]
    status: String,
    #[schema(index = "text")]
    description: String,

    // Info fields.
    #[schema(not_null)]
    category: String,
    parent_id: Option<Uuid>, // tag.id, tag.namespace = {tag.namespace}, tag.category = {tag.category}

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

impl Model for Tag {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            ..Self::default()
        }
    }

    fn read_map(&mut self, data: Map) -> Validation {
        let mut validation = Validation::new();
        if let Some(result) = Validation::parse_uuid(data.get("id")) {
            match result {
                Ok(id) => self.id = id,
                Err(err) => validation.record_fail("id", err.to_string()),
            }
        }
        if let Some(name) = Validation::parse_string(data.get("name")) {
            self.name = name;
        }
        if self.name.is_empty() {
            validation.record_fail("name", "must be nonempty");
        }
        validation
    }
}
