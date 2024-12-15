use crate::SharedString;

/// The query order.
#[derive(Debug, Clone)]
pub struct QueryOrder {
    /// The sort field.
    field: SharedString,
    /// The sort order.
    descending: bool,
    /// Whether the nulls appear first or last.
    nulls_first: Option<bool>,
}

impl QueryOrder {
    /// Creates a new instance.
    #[inline]
    pub fn new(field: impl Into<SharedString>, descending: bool) -> Self {
        Self {
            field: field.into(),
            descending,
            nulls_first: None,
        }
    }

    /// Sets the nulls first.
    #[inline]
    pub fn set_nulls_first(&mut self) {
        self.nulls_first = Some(true);
    }

    /// Sets the nulls last.
    #[inline]
    pub fn set_nulls_last(&mut self) {
        self.nulls_first = Some(false);
    }

    /// Returns the sort field.
    #[inline]
    pub fn field(&self) -> &str {
        self.field.as_ref()
    }

    /// Returns `true` if the sort order is ascending.
    #[inline]
    pub fn is_ascending(&self) -> bool {
        !self.descending
    }

    /// Returns `true` if the sort order is descending.
    #[inline]
    pub fn is_descending(&self) -> bool {
        self.descending
    }

    /// Returns `true` if the nulls appear first.
    #[inline]
    pub fn nulls_first(&self) -> bool {
        self.nulls_first.is_some_and(|b| b)
    }

    /// Returns `true` if the nulls appear last.
    #[inline]
    pub fn nulls_last(&self) -> bool {
        self.nulls_first.is_some_and(|b| !b)
    }
}
