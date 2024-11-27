use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{DecodeRow, Entity, Model, ModelAccessor, ModelHooks, Schema};

/// The `tag` model.
#[derive(
    Debug,
    Clone,
    Default,
    Serialize,
    Deserialize,
    DecodeRow,
    Schema,
    ModelAccessor,
    ModelHooks,
    Model,
    Entity,
)]
#[serde(default)]
#[schema(auto_rename)]
pub struct Tag {
    // Basic fields.
    #[schema(primary_key, auto_increment, read_only)]
    id: i64,
    #[schema(not_null, comment = "Tag name")]
    name: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    description: String,

    // Info fields.
    #[schema(not_null, index_type = "hash", comment = "Tag category")]
    category: String,
    #[schema(snapshot, reference = "Tag", comment = "Optional parent tag")]
    parent_id: Option<i64>,

    // Extensions.
    #[schema(reserved)]
    extra: Map,

    // Revisions.
    #[schema(read_only, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
}
