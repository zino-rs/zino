use crate::{request::Validation, Map};

#[derive(Debug, Clone)]
/// A mutation type of the model.
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
                    if let Some(fields) = Validation::parse_array(value) {
                        if fields.is_empty() {
                            validation.record_fail("fields", "must be nonempty");
                        } else {
                            self.fields = fields;
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
            self.fields.retain(|field| {
                fields
                    .iter()
                    .any(|key| field == key || field.ends_with(&format!(" {key}")))
            })
        }
    }

    /// Removes the editable fields in the deny list.
    #[inline]
    pub fn deny_fields(&mut self, fields: &[&str]) {
        self.fields.retain(|field| {
            !fields
                .iter()
                .any(|key| field == key || field.ends_with(&format!(" {key}")))
        })
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

impl Default for Mutation {
    fn default() -> Self {
        Self {
            fields: Vec::new(),
            updates: Map::new(),
        }
    }
}
