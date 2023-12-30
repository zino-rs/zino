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
    if let Some(params) = params.filter(|_| sql.contains('#')) {
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

/// Regex for the prepared statement.
static STATEMENT_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\#\{\s*([a-zA-Z]+[\w\.]*)\s*\}")
        .expect("fail to create a regex for the prepared statement")
});

#[cfg(test)]
mod tests {
    use crate::{extension::JsonObjectExt, Map};

    #[test]
    fn it_formats_sql_query_params() {
        let query = "SELECT ${fields} FROM users WHERE name = 'alice' AND age >= #{age};";
        let mut params = Map::new();
        params.upsert("fields", "id, name, age");
        params.upsert("age", 18);

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
