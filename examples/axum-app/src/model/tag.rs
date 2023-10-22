use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{ModelAccessor, ModelHooks, Schema};

/// The `tag` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor, ModelHooks)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Tag {
    // Basic fields.
    #[schema(auto_increment, readonly)]
    id: i64,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[cfg(feature = "namespace")]
    #[schema(default_value = "Tag::model_namespace", index_type = "hash")]
    namespace: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null)]
    category: String,
    #[schema(reference = "Tag")]
    parent_id: Option<i64>, // tag.id, tag.namespace = {tag.namespace}, tag.category = {tag.category}

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
        Self::default()
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
        if let Some(category) = data.parse_string("category") {
            self.category = category.into_owned();
        }
        if let Some(result) = data.parse_i64("parent_id") {
            match result {
                Ok(parent_id) => self.parent_id = Some(parent_id),
                Err(err) => validation.record_fail("parent_id", err),
            }
        }
        validation
    }
}
