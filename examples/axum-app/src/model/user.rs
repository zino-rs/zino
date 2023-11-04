use super::Tag;
use serde::{Deserialize, Serialize};
use zino::prelude::*;
use zino_derive::{Model, ModelAccessor, ModelHooks, Schema};
use zino_model::user::JwtAuthService;

/// The `User` model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor, ModelHooks, Model)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct User {
    // Basic fields.
    #[schema(primary_key, auto_increment, readonly)]
    id: i64,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Inactive", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null, unique, writeonly, constructor = "AccessKeyId::new")]
    access_key_id: String,
    #[schema(not_null, unique, writeonly)]
    account: String,
    #[schema(not_null, writeonly)]
    password: String,
    mobile: String,
    email: String,
    avatar: String,
    #[schema(snapshot, nonempty)]
    roles: Vec<String>,
    #[schema(reference = "Tag")]
    tags: Vec<i64>,

    // Security.
    last_login_at: DateTime,
    last_login_ip: String,
    current_login_at: DateTime,
    current_login_ip: String,
    login_count: u32,

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

impl JwtAuthService<i64> for User {
    const LOGIN_AT_FIELD: Option<&'static str> = Some("current_login_at");
    const LOGIN_IP_FIELD: Option<&'static str> = Some("current_login_ip");
}
