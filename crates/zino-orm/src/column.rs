use super::query::QueryExt;
use convert_case::{Case, Casing};
use std::borrow::Cow;
use zino_core::{
    JsonValue,
    extension::JsonObjectExt,
    model::{Column, Query},
};

/// Encoding a column to be sent to the database.
pub trait EncodeColumn<DB> {
    /// Returns the corresponding column type in the database.
    fn column_type(&self) -> &str;

    /// Encodes a json value as a column value represented by a str.
    fn encode_value<'a>(&self, value: Option<&'a JsonValue>) -> Cow<'a, str>;

    /// Formats a string value for the column.
    fn format_value<'a>(&self, value: &'a str) -> Cow<'a, str>;

    /// Formats a column filter.
    fn format_filter(&self, key: &str, value: &JsonValue) -> String;
}

/// Extension trait for [`Column`].
pub(super) trait ColumnExt {
    /// Returns `true` if it is compatible with the given data type.
    fn is_compatible(&self, data_type: &str) -> bool;

    /// Returns the type annotation.
    fn type_annotation(&self) -> &'static str;

    /// Returns the field definition.
    fn field_definition(&self, primary_key_name: &str) -> String;

    /// Returns the constraints.
    fn constraints(&self) -> Vec<String>;
}

impl ColumnExt for Column<'_> {
    fn is_compatible(&self, data_type: &str) -> bool {
        let column_type = self.column_type();
        if column_type.eq_ignore_ascii_case(data_type) {
            return true;
        }

        let data_type = data_type.to_ascii_uppercase();
        match column_type {
            "INT" => data_type == "INTEGER",
            "SMALLINT UNSIGNED" | "SMALLSERIAL" => data_type == "SMALLINT",
            "INT UNSIGNED" | "SERIAL" => data_type == "INT",
            "BIGINT UNSIGNED" | "BIGSERIAL" => data_type == "BIGINT",
            "TEXT" => data_type == "VARCHAR",
            _ => {
                if cfg!(feature = "orm-postgres") && column_type.ends_with("[]") {
                    data_type == "ARRAY"
                } else if column_type.starts_with("TIMESTAMP") {
                    data_type.starts_with("TIMESTAMP")
                } else if column_type.starts_with("VARCHAR") {
                    matches!(
                        data_type.as_str(),
                        "TEXT" | "VARCHAR" | "CHARACTER VARYING" | "ENUM"
                    )
                } else {
                    false
                }
            }
        }
    }

    fn type_annotation(&self) -> &'static str {
        if cfg!(feature = "orm-postgres") {
            match self.column_type() {
                "UUID" => "::UUID",
                "BIGINT" | "BIGSERIAL" => "::BIGINT",
                "INT" | "SERIAL" => "::INT",
                "SMALLINT" | "SMALLSERIAL" => "::SMALLINT",
                _ => "::TEXT",
            }
        } else {
            ""
        }
    }

    fn field_definition(&self, primary_key_name: &str) -> String {
        let column_name = self
            .extra()
            .get_str("column_name")
            .unwrap_or_else(|| self.name());
        let column_field = Query::format_field(column_name);
        let column_type = self.column_type();
        let mut definition = format!("{column_field} {column_type}");
        if column_name == primary_key_name {
            definition += " PRIMARY KEY";
        }
        if let Some(value) = self.default_value() {
            if self.auto_increment() {
                definition += if cfg!(any(
                    feature = "orm-mariadb",
                    feature = "orm-mysql",
                    feature = "orm-tidb"
                )) {
                    " AUTO_INCREMENT"
                } else {
                    // PostgreSQL does not support `AUTO INCREMENT` and SQLite does not need it.
                    ""
                };
            } else if self.auto_random() {
                // Only TiDB supports this feature.
                definition += if cfg!(feature = "orm-tidb") {
                    " AUTO_RANDOM"
                } else {
                    ""
                };
            } else {
                let value = self.format_value(value);
                if cfg!(feature = "orm-sqlite") && value.contains('(') {
                    definition = format!("{definition} DEFAULT ({value})");
                } else {
                    definition = format!("{definition} DEFAULT {value}");
                }
            }
        } else if self.is_not_null() {
            definition += " NOT NULL";
        }
        definition
    }

    fn constraints(&self) -> Vec<String> {
        let mut constraints = Vec::new();
        let extra = self.extra();
        let column_name = self
            .extra()
            .get_str("column_name")
            .unwrap_or_else(|| self.name());
        if let Some(reference) = self
            .reference()
            .filter(|_| extra.contains_key("foreign_key"))
        {
            let column_field = Query::format_field(column_name);
            let parent_table = Query::format_field(reference.name());
            let parent_column_field = Query::format_field(reference.column_name());
            let mut constraint = format!(
                "FOREIGN KEY ({column_field}) REFERENCES {parent_table}({parent_column_field})"
            );
            if let Some(action) = extra.get_str("on_delete") {
                constraint.push_str(" ON DELETE ");
                constraint.push_str(&action.to_case(Case::Upper));
            }
            if let Some(action) = extra.get_str("on_update") {
                constraint.push_str(" ON UPDATE ");
                constraint.push_str(&action.to_case(Case::Upper));
            }
            constraints.push(constraint);
        }
        constraints
    }
}
