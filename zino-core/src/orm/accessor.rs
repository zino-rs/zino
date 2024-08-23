use super::{ModelHelper, Schema};
use crate::{
    bail,
    datetime::DateTime,
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::{ModelHooks, Mutation, Query},
    validation::Validation,
    warn, JsonValue, Map,
};
use std::fmt::Display;

/// Access model fields.
///
/// This trait can be derived by `zino_derive::ModelAccessor`.
pub trait ModelAccessor<K>: Schema<PrimaryKey = K>
where
    K: Default + Display + PartialEq,
{
    /// Returns the `id` field, i.e. the primary key.
    fn id(&self) -> &K;

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

    /// Returns the `extra` field.
    #[inline]
    fn extra(&self) -> Option<&Map> {
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

    /// Returns the `deleted_at` field.
    #[inline]
    fn deleted_at(&self) -> Option<DateTime> {
        None
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
    fn snapshot(&self) -> Map {
        let mut snapshot = Map::new();
        snapshot.upsert(Self::PRIMARY_KEY_NAME, self.primary_key_value());
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
    fn has_namespace_prefix(&self, namespace: &str) -> bool {
        self.namespace()
            .strip_prefix(namespace)
            .is_some_and(|s| s.is_empty() || s.starts_with(':'))
    }

    /// Returns `true` if `self` has the namespace suffix.
    #[inline]
    fn has_namespace_suffix(&self, namespace: &str) -> bool {
        self.namespace()
            .strip_suffix(namespace)
            .is_some_and(|s| s.is_empty() || s.ends_with(':'))
    }

    /// Returns `true` if the model has the specific visibility.
    #[inline]
    fn has_visibility(&self, visibility: &str) -> bool {
        self.visibility().eq_ignore_ascii_case(visibility)
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

    /// Returns `true` if the `visibility` is `Protected`.
    #[inline]
    fn is_protected(&self) -> bool {
        self.visibility().eq_ignore_ascii_case("Protected")
    }

    /// Returns `true` if the `visibility` is `Private`.
    #[inline]
    fn is_private(&self) -> bool {
        self.visibility().eq_ignore_ascii_case("Private")
    }

    /// Returns `true` if the model has the specific status.
    #[inline]
    fn has_status(&self, status: &str) -> bool {
        self.status().eq_ignore_ascii_case(status)
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

    /// Returns `true` if the `status` is `Archived`.
    #[inline]
    fn is_archived(&self) -> bool {
        self.status().eq_ignore_ascii_case("Archived")
    }

    /// Returns `true` if the `description` is nonempty.
    #[inline]
    fn has_description(&self) -> bool {
        !self.description().is_empty()
    }

    /// Returns a reference to the value corresponding to the key in `extra`.
    #[inline]
    fn get_extra_value(&self, key: &str) -> Option<&JsonValue> {
        self.extra()?.get(key)
    }

    /// Returns the next version for the model.
    #[inline]
    fn next_version(&self) -> u64 {
        self.version() + 1
    }

    /// Constructs the query filters for the model of the current version.
    fn current_version_filters(&self) -> Map {
        let mut filters = Map::new();
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
        let mut filters = Map::new();
        filters.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
        filters.upsert("version", self.next_version());
        filters
    }

    /// Constructs the mutation updates for the model of the next version.
    fn next_version_updates(&self) -> Map {
        let mut updates = Map::new();
        updates.upsert("updated_at", DateTime::now().format_timestamp());
        updates.upsert("version", self.next_version());
        updates
    }

    /// Constructs the `Mutation` for the model of the next version.
    fn next_version_mutation(&self, updates: &mut Map) -> Mutation {
        let mut mutation = Self::default_mutation();
        mutation.append_updates(updates);
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
        let mut filters = Map::new();
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
        let mut filters = Map::new();
        filters.upsert(Self::PRIMARY_KEY_NAME, self.id().to_string());
        filters.upsert("edition", self.next_edition());
        filters
    }

    /// Constructs the mutation updates for the model of the next edition.
    fn next_edition_updates(&self) -> Map {
        let mut updates = Map::new();
        updates.upsert("updated_at", DateTime::now().format_timestamp());
        updates.upsert("version", self.next_version());
        updates.upsert("edition", self.next_edition());
        updates
    }

    /// Constructs the `Mutation` for the model of the next edition.
    fn next_edition_mutation(&self, updates: &mut Map) -> Mutation {
        let mut mutation = Self::default_mutation();
        mutation.append_updates(updates);
        mutation.append_updates(&mut self.next_edition_updates());
        mutation
    }

    /// Constructs a `Mutation` for logically deleting the model.
    fn soft_delete_mutation(&self) -> Mutation {
        let mut mutation = Self::default_mutation();
        let mut updates = self.next_edition_updates();
        updates.upsert("status", "Deleted");
        mutation.append_updates(&mut updates);
        mutation
    }

    /// Constructs a `Mutation` for locking the model.
    fn lock_mutation(&self) -> Mutation {
        let mut mutation = Self::default_mutation();
        let mut updates = self.next_edition_updates();
        updates.upsert("status", "Locked");
        mutation.append_updates(&mut updates);
        mutation
    }

    /// Constructs a `Mutation` for archiving the model.
    fn archive_mutation(&self) -> Mutation {
        let mut mutation = Self::default_mutation();
        let mut updates = self.next_edition_updates();
        updates.upsert("status", "Archived");
        mutation.append_updates(&mut updates);
        mutation
    }

    /// Constructs a default snapshot `Query` for the model.
    fn default_snapshot_query() -> Query {
        let mut query = Query::default();
        let fields = [
            Self::PRIMARY_KEY_NAME,
            "name",
            "status",
            "updated_at",
            "version",
        ];
        query.allow_fields(&fields);
        query.deny_fields(Self::write_only_fields());
        query
    }

    /// Constructs a default list `Query` for the model.
    fn default_list_query() -> Query {
        let mut query = Query::default();
        let ignored_fields = [Self::write_only_fields(), &["extra"]].concat();
        query.allow_fields(Self::fields());
        query.deny_fields(&ignored_fields);
        query.add_filter("status", Map::from_entry("$ne", "Deleted"));
        query.order_desc("updated_at");
        query
    }

    /// Checks the constraints for the model.
    async fn check_constraints(&self) -> Result<Validation, Error> {
        let mut validation = Validation::new();
        if self.id() == &K::default() {
            validation.record(Self::PRIMARY_KEY_NAME, "should not be a default value");
        }
        Ok(validation)
    }

    /// Fetches the data of models seleted by the `Query`.
    async fn fetch(query: &Query) -> Result<Vec<Map>, Error> {
        let mut models = Self::find(query).await?;
        let translate_enabled = query.translate_enabled();
        for model in models.iter_mut() {
            Self::after_decode(model).await?;
            translate_enabled.then(|| Self::translate_model(model));
        }
        Ok(models)
    }

    /// Fetches the data of a model seleted by the primary key.
    async fn fetch_by_id(id: &K) -> Result<Map, Error> {
        let mut model = Self::find_by_id::<Map>(id)
            .await?
            .ok_or_else(|| warn!("404 Not Found: cannot find the model `{}`", id))?;
        Self::after_decode(&mut model).await?;
        Self::translate_model(&mut model);
        Ok(model)
    }

    /// Deletes a model of the primary key by setting the status as `Deleted`.
    async fn soft_delete_by_id(id: &K) -> Result<(), Error> {
        let mut model = Self::try_get_model(id).await?;
        let model_data = model.before_soft_delete().await?;

        let query = model.current_version_query();
        let mut mutation = model.soft_delete_mutation();
        let ctx = Self::update_one(&query, &mut mutation).await?;
        Self::after_soft_delete(&ctx, model_data).await?;
        Ok(())
    }

    /// Locks a model of the primary key by setting the status as `Locked`.
    async fn lock_by_id(id: &K) -> Result<(), Error> {
        let mut model = Self::try_get_model(id).await?;
        let model_data = model.before_lock().await?;

        let query = model.current_version_query();
        let mut mutation = model.lock_mutation();
        let ctx = Self::update_one(&query, &mut mutation).await?;
        Self::after_lock(&ctx, model_data).await?;
        Ok(())
    }

    /// Archives a model of the primary key by setting the status as `Archived`.
    async fn archive_by_id(id: &K) -> Result<(), Error> {
        let mut model = Self::try_get_model(id).await?;
        let model_data = model.before_archive().await?;

        let query = model.current_version_query();
        let mut mutation = model.archive_mutation();
        let ctx = Self::update_one(&query, &mut mutation).await?;
        Self::after_archive(&ctx, model_data).await?;
        Ok(())
    }

    /// Updates a model of the primary key using the json object.
    async fn update_by_id(
        id: &K,
        data: &mut Map,
        extension: Option<<Self as ModelHooks>::Extension>,
    ) -> Result<(Validation, Self), Error> {
        Self::before_extract().await?;

        let mut model = Self::try_get_model(id).await?;
        let version = model.version();
        if data.get_u64("version").is_some_and(|v| version != v) {
            bail!(
                "409 Conflict: there is a version conflict for the model `{}`",
                id
            );
        }
        Self::before_validation(data, extension.as_ref()).await?;

        let validation = model.read_map(data);
        if !validation.is_success() {
            return Ok((validation, model));
        }
        if let Some(extension) = extension {
            model.after_extract(extension).await?;
        }

        let validation = model.check_constraints().await?;
        if !validation.is_success() {
            return Ok((validation, model));
        }
        if model.is_deleted() {
            data.retain(|key, _value| key == "status");
        } else if model.is_locked() {
            data.retain(|key, _value| key == "visibility" || key == "status");
        } else if model.is_archived() {
            bail!("403 Forbidden: archived model `{}` can not be modified", id);
        }
        model.after_validation(data).await?;

        let query = model.current_version_query();
        let mut mutation = model.next_version_mutation(data);

        let model_data = model.before_update().await?;
        let ctx = Self::update_one(&query, &mut mutation).await?;
        if ctx.rows_affected() != Some(1) {
            bail!(
                "404 Not Found: there is no version `{}` for the model `{}`",
                version,
                id,
            );
        }
        Self::after_update(&ctx, model_data).await?;
        Ok((validation, model))
    }

    /// Generates random associations for the model.
    async fn random_associations() -> Result<Map, Error> {
        let mut associations = Map::new();
        let table_name = Self::table_name();
        for col in Self::columns() {
            if col.reference().is_some_and(|r| r.name() == table_name) {
                let col_name = col.name();
                let size = col.random_size();
                let values = Self::sample(size).await?;
                if col.is_array_type() {
                    associations.upsert(col_name, values);
                } else {
                    associations.upsert(col_name, values.first().cloned());
                }
            }
        }
        Ok(associations)
    }

    /// Attempts to generate a mocked model.
    async fn mock() -> Result<(Validation, Self), Error> {
        let mut data = Self::before_mock().await?;
        let mut associations = Self::random_associations().await?;
        data.append(&mut associations);
        for col in Self::columns() {
            if !col.has_attribute("constructor") {
                let value = col.mock_value();
                if !value.is_ignorable() {
                    data.upsert(col.name(), value);
                }
            }
        }
        Self::before_validation(&mut data, None).await?;

        let mut model = Self::new();
        let validation = model.read_map(&data);
        if !validation.is_success() {
            return Ok((validation, model));
        }

        let validation = model.check_constraints().await?;
        if !validation.is_success() {
            return Ok((validation, model));
        }
        model.after_validation(&mut data).await?;
        model.after_mock().await?;
        Ok((validation, model))
    }
}
