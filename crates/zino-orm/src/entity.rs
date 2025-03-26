use super::query::QueryExt;
use std::{
    fmt::{self, Display},
    marker::PhantomData,
};
use zino_core::model::{Model, Query};

/// An interface for the model entity.
pub trait Entity: Model {
    /// The column type.
    type Column: ModelColumn<Self>;

    /// The primary key column.
    const PRIMARY_KEY: Self::Column;

    /// Formats the column name.
    #[inline]
    fn format_column(col: &Self::Column) -> String {
        [Self::MODEL_NAME, ".", col.as_ref()].concat()
    }
}

/// An interface for the model column.
pub trait ModelColumn<E: Entity>: AsRef<str> + Display {
    /// Converts `self` into a column expression.
    fn into_column_expr(self) -> String;
}

/// A column computed dynamically based on other columns or expressions.
#[derive(Debug, Clone, PartialEq)]
pub struct DerivedColumn<E: Entity> {
    /// The column expression.
    expr: String,
    /// The phantom data.
    phantom: PhantomData<E>,
}

impl<E: Entity> DerivedColumn<E> {
    /// Creates a new instance.
    #[inline]
    pub fn new(expr: String) -> Self {
        Self {
            expr,
            phantom: PhantomData,
        }
    }

    /// Constructs an instance for the column alias.
    #[inline]
    pub fn alias(alias: &str) -> Self {
        Self::new(alias.to_owned())
    }

    /// Constructs an instance for extracting the year from a column.
    #[inline]
    pub fn year(col: E::Column) -> Self {
        let col_name = E::format_column(&col);
        let field = Query::format_field(&col_name);
        let expr = if cfg!(feature = "orm-sqlite") {
            format!("strftime('%Y', {field})")
        } else {
            format!("year({field})")
        };
        Self::new(expr)
    }

    /// Constructs an instance for extracting the date from a column.
    #[inline]
    pub fn date(col: E::Column) -> Self {
        let col_name = E::format_column(&col);
        let field = Query::format_field(&col_name);
        Self::new(format!("date({field})"))
    }
}

impl<E: Entity> AsRef<str> for DerivedColumn<E> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.expr.as_str()
    }
}

impl<E: Entity> Display for DerivedColumn<E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.expr.fmt(f)
    }
}

impl<E: Entity> ModelColumn<E> for DerivedColumn<E> {
    #[inline]
    fn into_column_expr(self) -> String {
        self.expr
    }
}
