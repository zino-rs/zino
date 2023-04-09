use crate::{extension::JsonObjectExt, request::Validation, Map};
use serde_json::Value;

#[derive(Debug, Clone, Default)]
/// A mutation type for models.
pub struct Mutation {
    // Editable fields.
    fields: Vec<String>,
    // Updates.
    updates: Map,
}

impl Mutation {
    /// Creates a new instance.
    #[inline]
    pub fn new(updates: Map) -> Self {
        Self {
            fields: Vec::new(),
            updates,
        }
    }

    /// Updates the mutation using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        let updates = &mut self.updates;
        for (key, value) in data {
            match key.as_str() {
                "fields" => {
                    if let Some(fields) = Validation::parse_str_array(value) {
                        if fields.is_empty() {
                            validation.record("fields", "must be nonempty");
                        } else {
                            self.fields = fields.into_iter().map(|s| s.to_owned()).collect();
                        }
                    }
                }
                _ => {
                    if !key.starts_with('$') && value != "" {
                        updates.insert(key.to_owned(), value.to_owned());
                    }
                }
            }
        }
        validation
    }

    /// Retains the editable fields in the allow list.
    /// If the editable fields are empty, it will be set to the list.
    #[inline]
    pub fn allow_fields(&mut self, fields: &[&str]) {
        if self.fields.is_empty() {
            self.fields = fields.iter().map(|&key| key.to_owned()).collect::<Vec<_>>();
        } else {
            self.fields
                .retain(|field| fields.iter().any(|key| field == key))
        }
    }

    /// Removes the editable fields in the deny list.
    #[inline]
    pub fn deny_fields(&mut self, fields: &[&str]) {
        self.fields
            .retain(|field| !fields.iter().any(|key| field == key))
    }

    /// Adds a key-value pair to the mutation updates.
    #[inline]
    pub fn add_update(&mut self, key: impl Into<String>, value: impl Into<Value>) {
        self.updates.upsert(key, value);
    }

    /// Moves all elements from the `updates` into `self`.
    #[inline]
    pub fn append_updates(&mut self, updates: &mut Map) {
        self.updates.append(updates);
    }

    /// Returns a reference to the editable fields.
    #[inline]
    pub fn fields(&self) -> &[String] {
        self.fields.as_slice()
    }

    /// Returns a reference to the mutation updates.
    #[inline]
    pub fn updates(&self) -> &Map {
        &self.updates
    }
}
