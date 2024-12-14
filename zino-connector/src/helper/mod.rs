/// Helper utilities.
mod query;

#[cfg(any(
    feature = "connector-mysql",
    feature = "connector-sqlite",
    feature = "connector-postgres",
))]
mod sql_query;

pub(crate) use query::format_query;

#[cfg(any(
    feature = "connector-mysql",
    feature = "connector-sqlite",
    feature = "connector-postgres",
))]
pub(crate) use sql_query::prepare_sql_query;
