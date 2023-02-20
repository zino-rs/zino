/// Generates SQL `SET` expressions.
use super::{column::ColumnExt, Schema};
use crate::model::Mutation;

/// Extension trait for [`Mutation`](crate::model::Mutation).
pub(super) trait MutationExt {
    /// Formats the update to generate SQL `SET` expression.
    fn format_update<M: Schema>(&self) -> String;
}

impl MutationExt for Mutation {
    fn format_update<M: Schema>(&self) -> String {
        let update = self.update();
        let fields = self.fields();
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
                                    let value = col.encode_value(Some(value));
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
                                    let value = col.encode_value(Some(value));
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
                                    let value = col.encode_value(Some(value));
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
                                    let value = col.encode_value(Some(value));
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
                                    let value = col.encode_value(Some(value));
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
                                    let value = col.encode_value(Some(value));
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
                                    let value = col.encode_value(Some(value));
                                    let mutation = format!("{key} = GREATEST({key}, {value})");
                                    mutations.push(mutation);
                                }
                            }
                        }
                    }
                    _ => {
                        if fields.contains(key) && let Some(col) = M::get_column(key) {
                            let value = col.encode_value(Some(value));
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
