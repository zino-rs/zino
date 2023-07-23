use crate::{JsonValue, Map};
use regex::{Captures, Regex};
use std::{borrow::Cow, sync::LazyLock};

/// Prepares the SQL query for binding parameters
/// (`?` for most SQL flavors and `$N` for PostgreSQL).
///
/// The parameter is represented as `${param}` or `#{param}`,
/// in which `param` can only contain restricted chracters `[a-zA-Z]+[\w\.]*`.
pub(crate) fn prepare_sql_query<'a>(
    query: &'a str,
    params: Option<&'a Map>,
    placeholder: char,
) -> (Cow<'a, str>, Vec<&'a JsonValue>) {
    let sql = super::format_query(query, params);
    if let Some(params) = params && sql.contains('#') {
        let mut values = Vec::new();
        let sql = STATEMENT_PATTERN.replace_all(&sql, |captures: &Captures| {
            let key = &captures[1];
            let value = params.get(key).unwrap_or(&JsonValue::Null);
            values.push(value);
            if placeholder == '$' {
                Cow::Owned(format!("${}", values.len()))
            } else {
                Cow::Borrowed("?")
            }
        });
        (sql.into_owned().into(), values)
    } else {
        (sql, Vec::new())
    }
}

static STATEMENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\#\{\s*([a-zA-Z]+[\w\.]*)\s*\}").expect("fail to create the query pattern")
});
