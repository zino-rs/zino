use crate::{
    database::{ColumnExt, Schema},
    request::Validation,
    Map,
};

#[derive(Debug, Clone, Default)]
/// SQL mutation builder.
pub struct Mutation {
    // Editable fields.
    fields: Vec<String>,
    // Update.
    update: Map,
}

impl Mutation {
    /// Creates a new instance.
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            update: Map::new(),
        }
    }

    /// Updates the mutation using the json object and returns the validation result.
    #[must_use]
    pub fn read_map(&mut self, data: Map) -> Validation {
        let mut validation = Validation::new();
        let update = &mut self.update;
        for (key, value) in data.into_iter() {
            match key.as_str() {
                "fields" => {
                    if let Some(fields) = Validation::parse_array(&value) {
                        if fields.is_empty() {
                            validation.record_fail("fields", "must be nonempty");
                        } else {
                            self.fields = fields;
                        }
                    }
                }
                _ => {
                    if !key.starts_with('$') && value != "" {
                        update.insert(key, value);
                    }
                }
            }
        }
        validation
    }

    /// Retains the editable fields in the allow list of columns.
    /// If the editable fields are empty, it will be set to the allow list.
    #[inline]
    pub fn allow_fields<const N: usize>(&mut self, columns: [&str; N]) {
        let fields = &mut self.fields;
        if fields.is_empty() {
            self.fields = columns.map(|col| col.to_owned()).to_vec();
        } else {
            fields.retain(|field| {
                columns
                    .iter()
                    .any(|col| field == col || field.ends_with(&format!(" {col}")))
            })
        }
    }

    /// Removes the editable fields in the deny list of columns.
    #[inline]
    pub fn deny_fields<const N: usize>(&mut self, columns: [&str; N]) {
        self.fields.retain(|field| {
            !columns
                .iter()
                .any(|col| field == col || field.ends_with(&format!(" {col}")))
        })
    }

    /// Appends to the update.
    #[inline]
    pub fn append_update(&mut self, update: &mut Map) {
        self.update.append(update);
    }

    /// Returns a reference to the editable fields.
    #[inline]
    pub fn fields(&self) -> &[String] {
        self.fields.as_slice()
    }

    /// Formats the update to generate SQL `SET` expression.
    pub(crate) fn format_update<M: Schema>(&self) -> String {
        let update = &self.update;
        let fields = &self.fields;
        if update.is_empty() || fields.is_empty() {
            return String::new();
        }

        let mut mutations = Vec::new();
        for (key, value) in update.iter() {
            match key.as_str() {
                "$append" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = {key} || {value}");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                "$preppend" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = {value} || {key}");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                "$pull" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = array_remove({key}, {value})");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                "$inc" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = {key} + {value}");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                "$mul" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = {key} * {value}");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                "$min" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = LEAST({key}, {value})");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                "$max" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if fields.contains(key) && let Some(col) = M::get_column(key) {
                                let value = col.encode_value(value);
                                let mutation = format!("{key} = GREATEST({key}, {value})");
                                mutations.push(mutation);
                            }
                        }
                    }
                }
                _ => {
                    if fields.contains(key) && let Some(col) = M::get_column(key) {
                        let value = col.encode_value(value);
                        let mutation = format!("{key} = {value}");
                        mutations.push(mutation);
                    }
                }
            }
        }
        if mutations.is_empty() {
            String::new()
        } else {
            "SET ".to_owned() + &mutations.join(",")
        }
    }
}
