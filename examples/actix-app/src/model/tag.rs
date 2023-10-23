use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{ModelAccessor, ModelHooks, Schema};

/// The `tag` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor, ModelHooks)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Tag {
    // Basic fields.
    #[schema(primary_key, readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null)]
    category: String,
    #[schema(reference = "Tag")]
    parent_id: Option<Uuid>,

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

impl Model for Tag {
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
        if let Some(category) = data.parse_string("category") {
            self.category = category.into_owned();
        }
        if let Some(result) = data.parse_uuid("parent_id") {
            match result {
                Ok(parent_id) => self.parent_id = Some(parent_id),
                Err(err) => validation.record_fail("parent_id", err),
            }
        }
        validation
    }
}
