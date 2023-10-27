use crate::model::{Column, EncodeColumn};

/// Returns the column definition.
pub(super) fn column_def(col: &Column, primary_key_name: &str) -> String {
    let column_name = col.name();
    let column_type = col.column_type();
    let mut definition = format!("{column_name} {column_type}");
    if column_name == primary_key_name {
        definition += " PRIMARY KEY";
    }
    if let Some(value) = col.default_value() {
        if col.auto_increment() {
            definition += if cfg!(feature = "orm-mysql") {
                " AUTO_INCREMENT"
            } else {
                // PostgreSQL does not support `AUTO INCREMENT` and SQLite does not need it.
                ""
            };
        } else {
            let value = col.format_value(value);
            if cfg!(feature = "orm-sqlite") && value.contains('(') {
                definition = format!("{definition} DEFAULT ({value})");
            } else {
                definition = format!("{definition} DEFAULT {value}");
            }
        }
    } else if col.is_not_null() {
        definition += " NOT NULL";
    }
    definition
}
