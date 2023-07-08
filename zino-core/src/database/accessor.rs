use super::Schema;
use crate::{
    crypto,
    datetime::DateTime,
    encoding::base64,
    error::Error,
    extension::{JsonObjectExt, TomlTableExt},
    model::{Mutation, Query},
    request::Validation,
    state::State,
    JsonValue, Map,
};
use hkdf::Hkdf;
use sha2::{Digest, Sha256};
use std::{fmt::Display, sync::LazyLock};

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

    /// Returns `true` if the `description` is nonempty.
    #[inline]
    fn has_description(&self) -> bool {
        !self.description().is_empty()
    }

    /// Returns a reference to the value corresponding to the key in `content`.
    #[inline]
    fn get_content_value(&self, key: &str) -> Option<&JsonValue> {
        self.content()?.get(key)
    }

    /// Returns a reference to the value corresponding to the key in `extra`.
    #[inline]
    fn get_extra_value(&self, key: &str) -> Option<&JsonValue> {
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

    /// Constructs the `Mutation` for a lock of the model.
    fn lock_mutation(&self) -> Mutation {
        let mut mutation = Self::default_mutation();
        let mut updates = self.next_edition_updates();
        updates.upsert("status", "Locked");
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
        query.set_sort_order("updated_at", false);
        query
    }

    /// Returns the secret key for the model.
    /// It should have at least 64 bytes.
    ///
    /// # Note
    ///
    /// This should only be used for internal services. Do not expose it to external users.
    #[inline]
    fn secret_key() -> &'static [u8] {
        SECRET_KEY.as_slice()
    }

    /// Encrypts the password for the model.
    fn encrypt_password(passowrd: &str) -> Result<String, Error> {
        let key = Self::secret_key();
        let passowrd = passowrd.as_bytes();
        if let Ok(bytes) = base64::decode(passowrd) && bytes.len() == 256 {
            crypto::encrypt_hashed_password(key, passowrd)
        } else {
            crypto::encrypt_raw_password::<Sha256>(key, passowrd)
        }
    }

    /// Verifies the password for the model.
    fn verify_password(passowrd: &str, encrypted_password: &str) -> Result<bool, Error> {
        let key = Self::secret_key();
        let passowrd = passowrd.as_bytes();
        let encrypted_password = encrypted_password.as_bytes();
        if let Ok(bytes) = base64::decode(passowrd) && bytes.len() == 256 {
            crypto::verify_hashed_password(key, passowrd, encrypted_password)
        } else {
            crypto::verify_raw_password::<Sha256>(key, passowrd, encrypted_password)
        }
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
        let mut models = Self::find(query).await?;
        for model in models.iter_mut() {
            Self::after_decode(model).await?;
        }
        Ok(models)
    }

    /// Fetches the data of a model seleted by the primary key.
    async fn fetch_by_id(id: &T) -> Result<Map, Error> {
        let mut model = Self::find_by_id::<Map>(id)
            .await?
            .ok_or_else(|| Error::new(format!("404 Not Found: cannot find the model `{id}`")))?;
        Self::after_decode(&mut model).await?;
        Ok(model)
    }

    /// Updates a model of the primary key using the json object.
    async fn update_by_id(id: &T, mut data: Map) -> Result<(Validation, Self), Error> {
        let mut model = Self::try_get_model(id).await?;
        if let Some(version) = data.get_u64("version") && model.version() != version {
            return Err(Error::new("409 Conflict: there is a version control conflict"));
        }
        Self::before_validation(&mut data).await?;

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
        Self::after_validation(&mut data).await?;

        let query = model.current_version_query();
        let mutation = model.next_version_mutation(data);

        let model_data = model.before_update().await?;
        let ctx = Self::update_one(&query, &mutation).await?;
        Self::after_update(&ctx, model_data).await?;
        Ok((validation, model))
    }

    /// Deletes a model of the primary key by setting the status as `Deleted`.
    async fn soft_delete_by_id(id: &T) -> Result<(), Error> {
        let mut model = Self::try_get_model(id).await?;
        let model_data = model.before_soft_delete().await?;
        let query = model.current_version_query();
        let mutation = model.soft_delete_mutation();
        let ctx = Self::update_one(&query, &mutation).await?;
        Self::after_soft_delete(&ctx, model_data).await?;
        Ok(())
    }

    /// Locks a model of the primary key by setting the status as `Locked`.
    async fn lock_by_id(id: &T) -> Result<(), Error> {
        let mut model = Self::try_get_model(id).await?;
        let model_data = model.before_lock().await?;
        let query = model.current_version_query();
        let mutation = model.lock_mutation();
        let ctx = Self::update_one(&query, &mutation).await?;
        Self::after_lock(&ctx, model_data).await?;
        Ok(())
    }
}

/// Secret key.
static SECRET_KEY: LazyLock<[u8; 64]> = LazyLock::new(|| {
    let config = State::shared()
        .get_config("database")
        .expect("the `database` field should be a table");
    let database_checksum: [u8; 32] = config
        .get_str("checksum")
        .and_then(|checksum| checksum.as_bytes().try_into().ok())
        .unwrap_or_else(|| {
            let database_key = format!("{}_{}", *super::NAMESPACE_PREFIX, super::DRIVER_NAME);
            let mut hasher = Sha256::new();
            hasher.update(database_key.as_bytes());
            hasher.finalize().into()
        });

    let mut secret_key = [0; 64];
    let info = "ZINO:ORM;CHECKSUM:SHA256;HKDF:HMAC-SHA256";
    Hkdf::<Sha256>::from_prk(&database_checksum)
        .expect("pseudorandom key is not long enough")
        .expand(info.as_bytes(), &mut secret_key)
        .expect("invalid length for Sha256 to output");
    secret_key
});
