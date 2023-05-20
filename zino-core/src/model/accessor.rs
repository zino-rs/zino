use crate::{
    database::Schema,
    datetime::DateTime,
    extension::JsonObjectExt,
    model::{Mutation, Query},
    Map,
};
use serde_json::Value;
use std::fmt::Display;

/// Access model fields.
pub trait ModelAccessor<T, U>: Schema
where
    T: Default + Display + PartialEq,
    U: Default + Display + PartialEq,
{
    /// Returns the `id` field.
    fn id(&self) -> T;

    /// Returns the `name` field.
    #[inline]
    fn name(&self) -> &str {
        ""
    }

    /// Returns the `namespace` field.
    #[inline]
    fn namespace(&self) -> &str {
        ""
    }

    /// Returns the `visibility` field.
    #[inline]
    fn visibility(&self) -> &str {
        "Private"
    }

    /// Returns the `status` field.
    #[inline]
    fn status(&self) -> &str {
        "Active"
    }

    /// Returns the `description` field.
    #[inline]
    fn description(&self) -> &str {
        ""
    }

    /// Returns the `content` field.
    #[inline]
    fn content(&self) -> Option<&Map> {
        None
    }

    /// Returns the `extra` field.
    #[inline]
    fn extra(&self) -> Option<&Map> {
        None
    }

    /// Returns the `owner_id` field.
    #[inline]
    fn owner_id(&self) -> U {
        U::default()
    }

    /// Returns the `maintainer_id` field.
    #[inline]
    fn maintainer_id(&self) -> U {
        U::default()
    }

    /// Returns the `created_at` field.
    #[inline]
    fn created_at(&self) -> DateTime {
        DateTime::default()
    }

    /// Returns the `updated_at` field.
    #[inline]
    fn updated_at(&self) -> DateTime {
        DateTime::default()
    }

    /// Returns the `version` field.
    #[inline]
    fn version(&self) -> u64 {
        0
    }

    /// Returns the `edition` field.
    #[inline]
    fn edition(&self) -> u32 {
        0
    }

    /// Returns `true` if `self` has the namespace prefix.
    #[inline]
    fn has_namespace(&self, namespace: &str) -> bool {
        self.namespace()
            .strip_prefix(namespace)
            .is_some_and(|s| s.is_empty() || s.starts_with(':'))
    }

    /// Returns `true` if the `visibility` is `Public`.
    #[inline]
    fn is_public(&self) -> bool {
        self.visibility().eq_ignore_ascii_case("Public")
    }

    /// Returns `true` if the `visibility` is `Internal`.
    #[inline]
    fn is_internal(&self) -> bool {
        self.visibility().eq_ignore_ascii_case("Internal")
    }

    /// Returns `true` if the `visibility` is `Private`.
    #[inline]
    fn is_private(&self) -> bool {
        self.visibility().eq_ignore_ascii_case("Private")
    }

    /// Returns `true` if the `status` is `Active`.
    #[inline]
    fn is_active(&self) -> bool {
        self.status().eq_ignore_ascii_case("Active")
    }

    /// Returns `true` if the `status` is `Inactive`.
    #[inline]
    fn is_inactive(&self) -> bool {
        self.status().eq_ignore_ascii_case("Inactive")
    }

    /// Returns `true` if the `status` is `Locked`.
    #[inline]
    fn is_locked(&self) -> bool {
        self.status().eq_ignore_ascii_case("Locked")
    }

    /// Returns `true` if the `status` is `Deleted`.
    #[inline]
    fn is_deleted(&self) -> bool {
        self.status().eq_ignore_ascii_case("Deleted")
    }

    /// Returns a reference to the value corresponding to the key in `content`.
    #[inline]
    fn get_content_value(&self, key: &str) -> Option<&Value> {
        self.content()?.get(key)
    }

    /// Returns a reference to the value corresponding to the key in `extra`.
    #[inline]
    fn get_extra_value(&self, key: &str) -> Option<&Value> {
        self.extra()?.get(key)
    }

    /// Returns `true` if the `owner_id` is not the default.
    #[inline]
    fn has_owner(&self) -> bool {
        self.owner_id() != U::default()
    }

    /// Returns `true` if the `maintainer_id` is not the default.
    #[inline]
    fn has_maintainer(&self) -> bool {
        self.maintainer_id() != U::default()
    }

    /// Returns the next version for the model.
    #[inline]
    fn next_version(&self) -> u64 {
        self.version() + 1
    }

    /// Constructs the query filters for the model of the current version.
    fn current_version_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("version", self.version());
        filters
    }

    /// Constructs the `Query` for the model of the current version.
    fn current_version_query(&self) -> Query {
        let mut query = Self::default_query();
        query.append_filters(&mut self.current_version_filters());
        query
    }

    /// Constructs the query filters for the model of the next version.
    fn next_version_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("version", self.next_version());
        filters
    }

    /// Constructs the mutation updates for the model of the next version.
    fn next_version_updates(&self) -> Map {
        let mut updates = Map::with_capacity(2);
        updates.upsert("updated_at", DateTime::now().to_string());
        updates.upsert("version", self.next_version());
        updates
    }

    /// Constructs the `Mutation` for the model of the next version.
    fn next_version_mutation(&self, mut updates: Map) -> Mutation {
        let mut mutation = Self::default_mutation();
        mutation.append_updates(&mut updates);
        mutation.append_updates(&mut self.next_version_updates());
        mutation
    }

    /// Returns the next edition for the model.
    #[inline]
    fn next_edition(&self) -> u32 {
        self.edition() + 1
    }

    /// Constructs the query filters for the model of the current edition.
    fn current_edition_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("edition", self.edition());
        filters
    }

    /// Constructs the `Query` for the model of the current edition.
    fn current_edition_query(&self) -> Query {
        let mut query = Self::default_query();
        query.append_filters(&mut self.current_edition_filters());
        query
    }

    /// Constructs the query filters for the model of the next edition.
    fn next_edition_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert("id", self.id().to_string());
        filters.upsert("edition", self.next_edition());
        filters
    }

    /// Constructs the mutation updates for the model of the next edition.
    fn next_edition_updates(&self) -> Map {
        let mut updates = Map::with_capacity(2);
        updates.upsert("updated_at", DateTime::now().to_string());
        updates.upsert("version", self.next_version());
        updates.upsert("edition", self.next_edition());
        updates
    }

    /// Constructs the `Mutation` for the model of the next edition.
    fn next_edition_mutation(&self, mut updates: Map) -> Mutation {
        let mut mutation = Self::default_mutation();
        mutation.append_updates(&mut updates);
        mutation.append_updates(&mut self.next_edition_updates());
        mutation
    }

    /// Constructs the `Mutation` for a soft delete of the model.
    fn soft_delete_mutation(&self) -> Mutation {
        let mut mutation = Self::default_mutation();
        let mut updates = self.next_edition_updates();
        updates.upsert("status", "Deleted");
        mutation.append_updates(&mut updates);
        mutation
    }

    /// Constructs a default list `Query` for the model.
    #[inline]
    fn default_list_query() -> Query {
        let mut query = Self::default_query();
        query.add_filter("status", Map::from_entry("$ne", "Deleted"));
        query.set_sort_order("updated_at".to_owned(), false);
        query
    }
}
