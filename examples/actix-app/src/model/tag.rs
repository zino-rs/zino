use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{DecodeRow, Model, ModelAccessor, ModelHooks, Schema};

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
)]
#[serde(default)]
pub struct Tag {
    // Basic fields.
    #[schema(primary_key, read_only, constructor = "Uuid::now_v7")]
    id: Uuid,
    #[schema(not_null, comment = "Tag name")]
    name: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    description: String,

    // Info fields.
    #[schema(not_null, index_type = "hash", comment = "Tag category")]
    category: String,
    #[schema(snapshot, reference = "Tag", comment = "Optional parent tag")]
    parent_id: Option<Uuid>,

    // Extensions.
    #[schema(reserved)]
    content: Map,
    #[schema(reserved)]
    extra: Map,

    // Revisions.
    #[schema(read_only, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
}
