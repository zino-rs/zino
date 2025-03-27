use super::{IntoSqlValue, query::QueryExt};
use std::{
    fmt::{self, Display},
    marker::PhantomData,
};
use zino_core::{
    JsonValue,
    model::{Model, Query},
};

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

    /// Constructs an instance using `COALESCE` to provide a default value for the column.
    pub fn coalesce<V: IntoSqlValue>(col: E::Column, value: V) -> Self {
        let col_name = E::format_column(&col);
        let field = Query::format_field(&col_name);
        let expr = match value.into_sql_value() {
            JsonValue::Null => format!("coalesce({field}, NULL)"),
            JsonValue::Bool(b) => {
                if b {
                    format!("coalesce({field}, TRUE)")
                } else {
                    format!("coalesce({field}, FALSE)")
                }
            }
            JsonValue::Number(n) => {
                format!("coalesce({field}, {n})")
            }
            JsonValue::String(s) => {
                let value = Query::escape_string(s);
                format!("coalesce({field}, {value})")
            }
            value => {
                let value = Query::escape_string(value);
                format!("coalesce({field}, {value})")
            }
        };
        Self::new(expr)
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

    /// Constructs an instance for extracting the year-month from a column.
    #[inline]
    pub fn year_month(col: E::Column) -> Self {
        let col_name = E::format_column(&col);
        let field = Query::format_field(&col_name);
        let expr = if cfg!(any(
            feature = "orm-mariadb",
            feature = "orm-mysql",
            feature = "orm-tidb"
        )) {
            format!("date_format({field}, '%Y-%m')")
        } else if cfg!(feature = "orm-postgres") {
            format!("to_char({field}, 'YYYY-MM')")
        } else {
            format!("strftime('%Y-%m', {field})")
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

    /// Constructs an instance for formating a date-time as `%Y-%m-%d %H:%M:%S`.
    #[inline]
    pub fn format_date_time(col: E::Column) -> Self {
        let col_name = E::format_column(&col);
        let field = Query::format_field(&col_name);
        let expr = if cfg!(any(
            feature = "orm-mariadb",
            feature = "orm-mysql",
            feature = "orm-tidb"
        )) {
            format!("date_format({field}, '%Y-%m-%d %H:%i:%s')")
        } else if cfg!(feature = "orm-postgres") {
            format!("to_char({field}, 'YYYY-MM-DD HH24:MI:SS')")
        } else {
            format!("strftime('%Y-%m-%d %H:%M:%S', {field})")
        };
        Self::new(expr)
    }

    /// Constructs an instance for extracting values from a JSON column.
    #[inline]
    pub fn json_extract(col: E::Column, path: &str) -> Self {
        let col_name = E::format_column(&col);
        let field = Query::format_field(&col_name);
        let expr = if cfg!(feature = "orm-postgres") {
            let path = path.strip_prefix("$.").unwrap_or(path).replace('.', ", ");
            format!(r#"({field} #>> '{{{path}}}')"#)
        } else {
            format!(r#"json_unquote(json_extract({field}, '{path}'))"#)
        };
        Self::new(expr)
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
