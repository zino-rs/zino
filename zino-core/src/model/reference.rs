use serde::Serialize;

/// A model reference for a column.
#[derive(Debug, Clone, Serialize)]
pub struct Reference<'a> {
    /// Reference name, i.e. the table name.
    name: &'a str,
    /// Column name.
    column_name: &'a str,
}

impl<'a> Reference<'a> {
    /// Creates a new instance.
    #[inline]
    pub fn new(name: &'a str, column_name: &'a str) -> Self {
        Self { name, column_name }
    }

    /// Returns the reference name.
    #[inline]
    pub fn name(&self) -> &'a str {
        self.name
    }

    /// Returns the referenced column name.
    #[inline]
    pub fn column_name(&self) -> &'a str {
        self.column_name
    }
}
