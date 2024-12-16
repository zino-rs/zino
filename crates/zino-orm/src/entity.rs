use std::fmt::Display;
use zino_core::model::Model;

/// An interface for the model entity.
pub trait Entity: Model {
    /// The column type.
    type Column: AsRef<str> + Display;

    /// The primary key column.
    const PRIMARY_KEY: Self::Column;

    /// Formats the column name.
    #[inline]
    fn format_column(col: &Self::Column) -> String {
        [Self::MODEL_NAME, ".", col.as_ref()].concat()
    }
}
