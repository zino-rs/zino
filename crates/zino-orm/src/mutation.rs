/// Generates SQL `SET` expressions.
use super::{DatabaseDriver, EncodeColumn, Entity, IntoSqlValue, Schema, query::QueryExt};
use std::marker::PhantomData;
use zino_core::{
    JsonValue, Map,
    datetime::DateTime,
    extension::JsonObjectExt,
    model::{Mutation, Query},
};

/// A mutation builder for the model entity.
///
/// # Examples
/// ```rust,ignore
/// use crate::model::{User, UserColumn};
/// use zino_orm::{MutationBuilder, QueryBuilder, Schema};
///
/// let query = QueryBuilder::new()
///     .primary_key("01936dc6-e48c-7d22-8e69-b29f85682fac")
///     .and_not_in(UserColumn::Status, ["Deleted", "Locked"])
///     .build();
/// let mut mutation = MutationBuilder::new()
///     .set(UserColumn::Status, "Active")
///     .set_now(UserColumn::UpdatedAt)
///     .inc_one(UserColumn::Version)
///     .build();
/// let ctx = User::update_one(&query, &mut mutation).await?;
/// ```
#[derive(Debug, Clone)]
pub struct MutationBuilder<E: Entity> {
    /// The mutation updates.
    updates: Map,
    /// `$inc` operations.
    inc_ops: Map,
    /// `$mul` operations.
    mul_ops: Map,
    /// `$min` operations.
    min_ops: Map,
    /// `$max` operations.
    max_ops: Map,
    /// The phantom data.
    phantom: PhantomData<E>,
}

impl<E: Entity> MutationBuilder<E> {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            updates: Map::new(),
            inc_ops: Map::new(),
            mul_ops: Map::new(),
            min_ops: Map::new(),
            max_ops: Map::new(),
            phantom: PhantomData,
        }
    }

    /// Sets the value of a column.
    #[inline]
    pub fn set(mut self, col: E::Column, value: impl IntoSqlValue) -> Self {
        self.updates.upsert(col.as_ref(), value.into_sql_value());
        self
    }

    /// Sets the value of a column if the value is not null.
    #[inline]
    pub fn set_if_not_null(mut self, col: E::Column, value: impl IntoSqlValue) -> Self {
        let value = value.into_sql_value();
        if !value.is_null() {
            self.updates.upsert(col.as_ref(), value);
        }
        self
    }

    /// Sets the value of a column if the value is not none.
    #[inline]
    pub fn set_if_some<T: IntoSqlValue>(mut self, col: E::Column, value: Option<T>) -> Self {
        if let Some(value) = value {
            self.updates.upsert(col.as_ref(), value.into_sql_value());
        }
        self
    }

    /// Sets the value of a column to null.
    #[inline]
    pub fn set_null(mut self, col: E::Column) -> Self {
        self.updates.upsert(col.as_ref(), JsonValue::Null);
        self
    }

    /// Sets the value of a column to the current date time.
    #[inline]
    pub fn set_now(mut self, col: E::Column) -> Self {
        self.updates
            .upsert(col.as_ref(), DateTime::now().into_sql_value());
        self
    }

    /// Increments the value of a column.
    #[inline]
    pub fn inc(mut self, col: E::Column, value: impl IntoSqlValue) -> Self {
        self.inc_ops.upsert(col.as_ref(), value.into_sql_value());
        self
    }

    /// Increments the value of a column by 1.
    #[inline]
    pub fn inc_one(mut self, col: E::Column) -> Self {
        self.inc_ops.upsert(col.as_ref(), 1);
        self
    }

    /// Multiplies the value of a column by a number.
    #[inline]
    pub fn mul(mut self, col: E::Column, value: impl IntoSqlValue) -> Self {
        self.mul_ops.upsert(col.as_ref(), value.into_sql_value());
        self
    }

    /// Updates the value of a column to a specified value
    /// if the specified value is less than the current value of the column.
    #[inline]
    pub fn min(mut self, col: E::Column, value: impl IntoSqlValue) -> Self {
        self.min_ops.upsert(col.as_ref(), value.into_sql_value());
        self
    }

    /// Updates the value of a column to a specified value
    /// if the specified value is greater than the current value of the column.
    #[inline]
    pub fn max(mut self, col: E::Column, value: impl IntoSqlValue) -> Self {
        self.max_ops.upsert(col.as_ref(), value.into_sql_value());
        self
    }

    /// Builds the model mutation.
    pub fn build(self) -> Mutation {
        let mut updates = self.updates;
        let inc_ops = self.inc_ops;
        let mul_ops = self.mul_ops;
        let min_ops = self.min_ops;
        let max_ops = self.max_ops;
        if !inc_ops.is_empty() {
            updates.upsert("$inc", inc_ops);
        }
        if !mul_ops.is_empty() {
            updates.upsert("$mul", mul_ops);
        }
        if !min_ops.is_empty() {
            updates.upsert("$min", min_ops);
        }
        if !max_ops.is_empty() {
            updates.upsert("$max", max_ops);
        }
        Mutation::new(updates)
    }
}

impl<E: Entity> Default for MutationBuilder<E> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for [`Mutation`](crate::model::Mutation).
pub(super) trait MutationExt<DB> {
    /// Formats the updates to generate SQL `SET` expression.
    fn format_updates<M: Schema>(&self) -> String;
}

impl MutationExt<DatabaseDriver> for Mutation {
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
                "$inc" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if permissive || fields.contains(key) {
                                if let Some(col) = M::get_writable_column(key) {
                                    let key = Query::format_field(key);
                                    let value = col.encode_value(Some(value));
                                    let mutation = format!(r#"{key} = {value} + {key}"#);
                                    mutations.push(mutation);
                                }
                            }
                        }
                    }
                }
                "$mul" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if permissive || fields.contains(key) {
                                if let Some(col) = M::get_writable_column(key) {
                                    let key = Query::format_field(key);
                                    let value = col.encode_value(Some(value));
                                    let mutation = format!(r#"{key} = {value} * {key}"#);
                                    mutations.push(mutation);
                                }
                            }
                        }
                    }
                }
                "$min" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if permissive || fields.contains(key) {
                                if let Some(col) = M::get_writable_column(key) {
                                    let key = Query::format_field(key);
                                    let value = col.encode_value(Some(value));
                                    let mutation = if cfg!(feature = "orm-sqlite") {
                                        format!(r#"{key} = MIN({value}, {key})"#)
                                    } else {
                                        format!(r#"{key} = LEAST({value}, {key})"#)
                                    };
                                    mutations.push(mutation);
                                }
                            }
                        }
                    }
                }
                "$max" => {
                    if let Some(update) = value.as_object() {
                        for (key, value) in update.iter() {
                            if permissive || fields.contains(key) {
                                if let Some(col) = M::get_writable_column(key) {
                                    let key = Query::format_field(key);
                                    let value = col.encode_value(Some(value));
                                    let mutation = if cfg!(feature = "orm-sqlite") {
                                        format!(r#"{key} = MAX({value}, {key})"#)
                                    } else {
                                        format!(r#"{key} = GREATEST({value}, {key})"#)
                                    };
                                    mutations.push(mutation);
                                }
                            }
                        }
                    }
                }
                _ => {
                    if permissive || fields.contains(key) {
                        if let Some(col) = M::get_writable_column(key) {
                            let key = Query::format_field(key);
                            let mutation = if let Some(subquery) =
                                value.as_object().and_then(|m| m.get_str("$subquery"))
                            {
                                format!(r#"{key} = {subquery}"#)
                            } else {
                                let value = col.encode_value(Some(value));
                                format!(r#"{key} = {value}"#)
                            };
                            mutations.push(mutation);
                        }
                    }
                }
            }
        }
        mutations.join(", ")
    }
}
