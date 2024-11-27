use super::Tag;
use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{DecodeRow, Entity, Model, ModelAccessor, ModelHooks, Schema};
use zino_model::user::JwtAuthService;

/// The `user` model.
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
pub struct User {
    // Basic fields.
    #[schema(primary_key, auto_increment, read_only)]
    id: i64,
    #[schema(not_null, comment = "User name")]
    name: String,
    #[schema(
        auto_initialized,
        enum_values = "Active | Inactive | Locked | Deleted | Archived",
        default_value = "Inactive",
        index_type = "hash",
        comment = "User status"
    )]
    status: String,
    description: String,

    // Info fields.
    #[schema(not_null, unique, write_only, constructor = "AccessKeyId::new")]
    access_key_id: String,
    #[schema(
        not_null,
        unique,
        write_only,
        index_type = "hash",
        min_length = 4,
        max_length = 16,
        comment = "User account"
    )]
    account: String,
    #[schema(not_null, write_only, comment = "User password")]
    password: String,
    mobile: String,
    #[schema(format = "email")]
    email: String,
    #[schema(format = "uri")]
    avatar: String,
    #[schema(
        snapshot,
        nonempty,
        unique_items,
        enum_values = "admin | worker | auditor",
        example = "admin, worker",
        comment = "User roles"
    )]
    roles: Vec<String>,
    #[schema(
        unique_items,
        reference = "Tag",
        fetch_as = "tags",
        comment = "User tags"
    )]
    tags: Vec<i64>,

    // Security.
    #[schema(generated)]
    last_login_at: DateTime,
    #[schema(generated, format = "ip")]
    last_login_ip: String,
    #[schema(generated)]
    current_login_at: DateTime,
    #[schema(generated, format = "ip")]
    current_login_ip: String,
    #[schema(generated)]
    login_count: u32,

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

impl JwtAuthService<i64> for User {
    const LOGIN_AT_FIELD: Option<&'static str> = Some("current_login_at");
    const LOGIN_IP_FIELD: Option<&'static str> = Some("current_login_ip");
}
