use super::Schema;
use crate::{
    datetime::DateTime,
    error::Error,
    extension::JsonObjectExt,
    model::{Mutation, Query},
    request::Validation,
    Map,
};
use serde_json::Value;
use std::fmt::Display;

/// Access model fields.
pub trait ModelAccessor<T, U = T>: Schema<PrimaryKey = T>
where
    T: Default + Display + PartialEq,
    U: Default + Display + PartialEq,
{
    /// Returns the `id` field, i.e. the primary key.
    fn id(&self) -> &T;

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
    fn owner_id(&self) -> Option<&U> {
        None
    }

    /// Returns the `maintainer_id` field.
    #[inline]
    fn maintainer_id(&self) -> Option<&U> {
        None
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

    /// Returns a snapshot of the model.
    #[inline]
    fn snapshot(&self) -> Map {
        let mut snapshot = Map::with_capacity(5);
        snapshot.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
        snapshot.upsert("name", self.name());
        snapshot.upsert("status", self.status());
        snapshot.upsert("updated_at", self.updated_at());
        snapshot.upsert("version", self.version());
        snapshot
    }

    /// Returns `true` if the `name` is nonempty.
    #[inline]
    fn has_name(&self) -> bool {
        !self.name().is_empty()
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

    /// Returns `true` if the `description` is nonempty.
    #[inline]
    fn has_description(&self) -> bool {
        !self.description().is_empty()
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
        self.owner_id()
            .is_some_and(|owner_id| owner_id != &U::default())
    }

    /// Returns `true` if the `maintainer_id` is not the default.
    #[inline]
    fn has_maintainer(&self) -> bool {
        self.maintainer_id()
            .is_some_and(|maintainer_id| maintainer_id != &U::default())
    }

    /// Returns the next version for the model.
    #[inline]
    fn next_version(&self) -> u64 {
        self.version() + 1
    }

    /// Constructs the query filters for the model of the current version.
    fn current_version_filters(&self) -> Map {
        let mut filters = Map::with_capacity(2);
        filters.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
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
        filters.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
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
        filters.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
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
        filters.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
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

    /// Constructs a default snapshot `Query` for the model.
    fn default_snapshot_query() -> Query {
        let mut query = Self::default_query();
        let fields = [
            Self::PRIMARY_KEY_NAME,
            "name",
            "status",
            "updated_at",
            "version",
        ];
        query.allow_fields(&fields);
        query
    }

    /// Constructs a default list `Query` for the model.
    fn default_list_query() -> Query {
        let mut query = Self::default_query();
        query.deny_fields(&["content", "extra"]);
        query.add_filter("status", Map::from_entry("$ne", "Deleted"));
        query.set_sort_order("updated_at".to_owned(), false);
        query
    }

    /// Filters the values of the primary key.
    async fn filter(primary_key_values: Vec<Value>) -> Result<Vec<Value>, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let limit = primary_key_values.len();
        let mut query = Query::default();
        query.allow_fields(&[primary_key_name]);
        query.add_filter(primary_key_name, Map::from_entry("$in", primary_key_values));
        query.add_filter("status", Map::from_entry("$ne", "Deleted"));
        query.set_limit(limit);

        let data = Self::find::<Map>(&query).await?;
        let mut primary_key_values = Vec::with_capacity(data.len());
        for map in data.into_iter() {
            for (_key, value) in map.into_iter() {
                primary_key_values.push(value);
            }
        }
        Ok(primary_key_values)
    }

    /// Checks the constraints for the model.
    async fn check_constraints(&self) -> Result<Validation, Error> {
        let mut validation = Validation::new();
        if self.id() == &T::default() {
            validation.record(Self::PRIMARY_KEY_NAME, "should not be a default value");
        }
        Ok(validation)
    }

    /// Fetches the data of models seleted by the `Query`.
    async fn fetch(query: &Query) -> Result<Vec<Map>, Error> {
        let models = Self::find(query).await?;
        Ok(models)
    }

    /// Fetches the data of a model seleted by the primary key.
    async fn fetch_by_id(id: &T) -> Result<Map, Error> {
        let model: Map = Self::find_by_id(id)
            .await?
            .ok_or_else(|| Error::new(format!("cannot find the model `{id}`")))?;
        Ok(model)
    }

    /// Updates a model of the primary key using the json object.
    async fn update_by_id(id: &T, mut data: Map) -> Result<(Validation, Self), Error> {
        let mut model = Self::try_get_model(id).await?;
        let validation = model.read_map(&data);
        if !validation.is_success() {
            return Ok((validation, model));
        }

        let validation = model.check_constraints().await?;
        if !validation.is_success() {
            return Ok((validation, model));
        }
        if model.is_locked() {
            data.retain(|key, _value| key == "visibility" || key == "status");
        } else if model.is_deleted() {
            data.retain(|key, _value| key == "status");
        }

        let query = model.current_version_query();
        let mutation = model.next_version_mutation(data);
        Self::update_one(&query, &mutation).await?;
        Ok((validation, model))
    }

    /// Deletes a model of the primary key by setting the status as `Deleted`.
    async fn soft_delete_by_id(id: &T) -> Result<(), Error> {
        let model = Self::try_get_model(id).await?;
        let query = model.current_version_query();
        let mutation = model.soft_delete_mutation();
        Self::update_one(&query, &mutation).await
    }
}
