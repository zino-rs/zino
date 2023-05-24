use crate::Map;
use regex::{Captures, Regex};
use serde_json::Value;
use std::{borrow::Cow, sync::LazyLock};

/// Formats the query using interpolation of the parameters.
///
/// The interpolation parameter is represented as `${param}`,
/// in which `param` can only contain restricted chracters `[a-zA-Z]+[\w\.]*`.
pub(crate) fn format_query<'a>(query: &'a str, params: Option<&'a Map>) -> Cow<'a, str> {
    if let Some(params) = params && query.contains('$') {
        INTERPOLATION_PATTERN.replace_all(query, |captures: &Captures| {
            let key = &captures[1];
            params
                .get(key)
                .map(|value| match value {
                    Value::String(s) => s.to_owned(),
                    _ => value.to_string(),
                })
                .unwrap_or_else(|| format!("${{{key}}}"))
        })
    } else {
        Cow::Borrowed(query)
    }
}

/// Prepares the SQL query for binding parameters
/// (`?` for most SQL flavors and `$N` for PostgreSQL).
///
/// The parameter is represented as `${param}` or `#{param}`,
/// in which `param` can only contain restricted chracters `[a-zA-Z]+[\w\.]*`.
#[cfg(any(
    feature = "connector-mssql",
    feature = "connector-mysql",
    feature = "connector-sqlite",
    feature = "connector-postgres",
    feature = "orm",
    feature = "orm-mysql",
    feature = "orm-postgres",
))]
pub(crate) fn prepare_sql_query<'a>(
    query: &'a str,
    params: Option<&'a Map>,
    placeholder: char,
) -> (Cow<'a, str>, Vec<&'a Value>) {
    let sql = format_query(query, params);
    if let Some(params) = params && sql.contains('#') {
        let mut values = Vec::new();
        let sql = STATEMENT_PATTERN.replace_all(&sql, |captures: &Captures| {
            let key = &captures[1];
            let value = params.get(key).unwrap_or(&Value::Null);
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

/// Interpolation pattern.
static INTERPOLATION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\$\{\s*([a-zA-Z]+[\w\.]*)\s*\}").expect("fail to create the interpolation pattern")
});

/// Statement pattern.
static STATEMENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\#\{\s*([a-zA-Z]+[\w\.]*)\s*\}").expect("fail to create the query pattern")
});

#[cfg(test)]
mod tests {
    use crate::{extension::JsonObjectExt, Map};

    #[test]
    fn it_formats_query_params() {
        let query = "SELECT ${fields} FROM users WHERE name = 'alice' AND age >= #{age};";
        let mut params = Map::new();
        params.upsert("fields", "id, name, age");
        params.upsert("age", 18);

        let sql = super::format_query(query, Some(&params));
        assert_eq!(
            sql,
            "SELECT id, name, age FROM users WHERE name = 'alice' AND age >= #{age};"
        );

        let (sql, values) = super::prepare_sql_query(query, Some(&params), '?');
        assert_eq!(
            sql,
            "SELECT id, name, age FROM users WHERE name = 'alice' AND age >= ?;"
        );
        assert_eq!(values[0], 18);

        let (sql, values) = super::prepare_sql_query(query, Some(&params), '$');
        assert_eq!(
            sql,
            "SELECT id, name, age FROM users WHERE name = 'alice' AND age >= $1;"
        );
        assert_eq!(values[0], 18);
    }
}
