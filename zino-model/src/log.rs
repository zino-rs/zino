use crate::User;
use serde::{Deserialize, Serialize};
use zino_core::{
    database::ModelHooks, datetime::DateTime, extension::JsonObjectExt, model::Model,
    request::Validation, Map, Uuid,
};
use zino_derive::{ModelAccessor, Schema};

/// The log model.
#[derive(Debug, Clone, Default, Serialize, Deserialize, Schema, ModelAccessor)]
#[serde(rename_all = "snake_case")]
#[serde(default)]
pub struct Log {
    // Basic fields.
    #[schema(readonly)]
    id: Uuid,
    #[schema(not_null, index_type = "text")]
    name: String,
    #[schema(default_value = "Log::model_namespace", index_type = "hash")]
    namespace: String,
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    #[schema(index_type = "text")]
    description: String,

    // Info fields.
    #[schema(not_null, readonly)]
    service: String,
    #[schema(readonly)]
    server_host: String,
    #[schema(readonly)]
    client_ip: String,
    topic: String,
    #[schema(readonly)]
    level: String,
    #[schema(readonly, index_type = "text")]
    message: String,
    #[schema(readonly)]
    source: String,
    #[schema(readonly, index_type = "btree")]
    recorded_at: DateTime,

    // Extensions.
    #[schema(readonly)]
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

impl Model for Log {
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
        validation
    }
}

impl ModelHooks for Log {}
