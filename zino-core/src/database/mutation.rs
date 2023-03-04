/// Generates SQL `SET` expressions.
use super::Schema;
use crate::model::{EncodeColumn, Mutation};
use sqlx::Postgres;

/// Extension trait for [`Mutation`](crate::model::Mutation).
pub(super) trait MutationExt<DB> {
    /// Formats the updates to generate SQL `SET` expression.
    fn format_updates<M: Schema>(&self) -> String;
}

impl MutationExt<Postgres> for Mutation {
    fn format_updates<M: Schema>(&self) -> String {
        let updates = self.updates();
        if updates.is_empty() {
            return String::new();
        }

        let fields = self.fields();
        let permissive = fields.is_empty();
        let mut mutations = Vec::new();
        for (key, value) in updates.iter() {
            match key.as_str() {
                    "$append" => {
                        if let Some(update) = value.as_object() {
                            for (key, value) in update.iter() {
                                if fields.contains(key) && let Some(col) = M::get_column(key) {
                                    let value = Postgres::encode_value(col, Some(value));
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
                                    let value = Postgres::encode_value(col, Some(value));
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
                                    let value = Postgres::encode_value(col, Some(value));
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
                                    let value = Postgres::encode_value(col, Some(value));
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
                                    let value = Postgres::encode_value(col, Some(value));
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
                                    let value = Postgres::encode_value(col, Some(value));
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
                                    let value = Postgres::encode_value(col, Some(value));
                                    let mutation = format!("{key} = GREATEST({key}, {value})");
                                    mutations.push(mutation);
                                }
                            }
                        }
                    }
                    _ => {
                        if (permissive || fields.contains(key)) && let Some(col) = M::get_column(key) {
                            let value = Postgres::encode_value(col, Some(value));
                            let mutation = format!("{key} = {value}");
                            mutations.push(mutation);
                        }
                    }
                }
        }
        mutations.join(", ")
    }
}
