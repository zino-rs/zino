//! The `tag` model and related services.

use serde::{Deserialize, Serialize};
use zino_core::{
    Map, Uuid,
    datetime::DateTime,
    error::Error,
    extension::JsonObjectExt,
    model::{Model, ModelHooks},
    validation::Validation,
};
use zino_derive::{DecodeRow, Entity, ModelAccessor, Schema};

#[cfg(any(feature = "owner-id", feature = "maintainer-id"))]
use crate::user::User;

#[cfg(feature = "maintainer-id")]
use zino_auth::UserSession;

/// The `tag` model.
#[derive(
    Debug, Clone, Default, Serialize, Deserialize, DecodeRow, Entity, Schema, ModelAccessor,
)]
#[serde(default)]
#[schema(auto_rename)]
pub struct Tag {
    // Basic fields.
    #[schema(read_only)]
    id: Uuid,
    #[schema(not_null)]
    name: String,
    #[cfg(feature = "namespace")]
    #[schema(default_value = "Tag::model_namespace", index_type = "hash")]
    namespace: String,
    #[cfg(feature = "visibility")]
    #[schema(default_value = "Internal")]
    visibility: String,
    #[schema(default_value = "Active", index_type = "hash")]
    status: String,
    description: String,

    // Info fields.
    #[schema(not_null)]
    category: String,
    #[schema(snapshot, reference = "Tag")]
    parent_id: Option<Uuid>, // tag.id, tag.namespace = {tag.namespace}, tag.category = {tag.category}

    // Extensions.
    extra: Map,

    // Revisions.
    #[cfg(feature = "owner-id")]
    #[schema(reference = "User")]
    owner_id: Option<Uuid>, // user.id
    #[cfg(feature = "maintainer-id")]
    #[schema(reference = "User")]
    maintainer_id: Option<Uuid>, // user.id
    #[schema(read_only, default_value = "now", index_type = "btree")]
    created_at: DateTime,
    #[schema(default_value = "now", index_type = "btree")]
    updated_at: DateTime,
    version: u64,
    #[cfg(feature = "edition")]
    edition: u32,
}

impl Model for Tag {
    const MODEL_NAME: &'static str = "tag";

    #[inline]
    fn new() -> Self {
        Self {
            id: Uuid::now_v7(),
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
        #[cfg(feature = "owner-id")]
        if let Some(result) = data.parse_uuid("owner_id") {
            match result {
                Ok(owner_id) => self.owner_id = Some(owner_id),
                Err(err) => validation.record_fail("owner_id", err),
            }
        }
        #[cfg(feature = "maintainer-id")]
        if let Some(result) = data.parse_uuid("maintainer_id") {
            match result {
                Ok(maintainer_id) => self.maintainer_id = Some(maintainer_id),
                Err(err) => validation.record_fail("maintainer_id", err),
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

impl ModelHooks for Tag {
    type Data = ();
    #[cfg(feature = "maintainer-id")]
    type Extension = UserSession<Uuid, String>;
    #[cfg(not(feature = "maintainer-id"))]
    type Extension = ();

    #[cfg(feature = "maintainer-id")]
    #[inline]
    async fn after_extract(&mut self, session: Self::Extension) -> Result<(), Error> {
        self.maintainer_id = Some(*session.user_id());
        Ok(())
    }

    #[cfg(feature = "maintainer-id")]
    #[inline]
    async fn before_validation(
        data: &mut Map,
        extension: Option<&Self::Extension>,
    ) -> Result<(), Error> {
        if let Some(session) = extension {
            data.upsert("maintainer_id", session.user_id().to_string());
        }
        Ok(())
    }
}
