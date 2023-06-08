use crate::User;
use serde::{Deserialize, Serialize};
use zino_core::{
    datetime::DateTime,
    extension::JsonObjectExt,
    model::{Model, ModelHooks},
    request::Validation,
    Map, Uuid,
};
use zino_derive::{ModelAccessor, Schema};

/// The tag model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Tag {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Tag::model_namespace", index_type = "hash")]
    namespace: String,
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null)]
    category: String,
    #[schema(reference = "Tag")]
    parent_id: Option<Uuid>, // tag.id, tag.namespace = {tag.namespace}, tag.category = {tag.category}

    // Extensions.
    content: Map,
    extra: Map,

    // Revisions.
    #[schema(reference = "User")]
    owner_id: Option<Uuid>, // user.id
    #[schema(reference = "User")]
    maintainer_id: Option<Uuid>, // user.id
    #[schema(readonly, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
    edition: u32,
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

impl Tag {
    /// Returns the `category`.
    #[inline]
    pub fn category(&self) -> &str {
        &self.category
    }

    /// Returns the `parent_id`.
    #[inline]
    pub fn parent_id(&self) -> Option<&Uuid> {
        self.parent_id
            .as_ref()
            .filter(|parent_id| !parent_id.is_nil())
    }
}

impl ModelHooks for Tag {}
