use super::{
    column::ColumnExt, mutation::MutationExt, query::QueryExt, ConnectionPool, DatabaseRow,
    Executor, GlobalPool, ModelHelper,
};
use crate::{
    bail,
    error::Error,
    extension::{JsonObjectExt, JsonValueExt},
    model::{Column, DecodeRow, EncodeColumn, ModelHooks, Mutation, Query, QueryContext},
    warn, JsonValue, Map,
};
use serde::de::DeserializeOwned;
use std::{fmt::Display, sync::atomic::Ordering::Relaxed};

/// Database schema.
///
/// This trait can be derived by `zino_derive::Schema`.
pub trait Schema: 'static + Send + Sync + ModelHooks {
    /// Primary key.
    type PrimaryKey: Default + Display + PartialEq;

    /// Primary key name.
    const PRIMARY_KEY_NAME: &'static str = "id";
    /// Reader name.
    const READER_NAME: &'static str = "main";
    /// Writer name.
    const WRITER_NAME: &'static str = "main";
    /// Optional custom table name.
    const TABLE_NAME: Option<&'static str> = None;

    /// Returns the primary key.
    fn primary_key(&self) -> &Self::PrimaryKey;

    /// Returns a reference to the Avro schema.
    fn schema() -> &'static apache_avro::Schema;

    /// Returns a reference to the columns.
    fn columns() -> &'static [Column<'static>];

    /// Returns a reference to the column fields.
    fn fields() -> &'static [&'static str];

    /// Returns a reference to the read-only column fields.
    fn read_only_fields() -> &'static [&'static str];

    /// Returns a reference to the write-only column fields.
    fn write_only_fields() -> &'static [&'static str];

    /// Retrieves a connection pool for the model reader.
    async fn acquire_reader() -> Result<&'static ConnectionPool, Error>;

    /// Retrieves a connection pool for the model writer.
    async fn acquire_writer() -> Result<&'static ConnectionPool, Error>;

    /// Returns the driver name.
    ///
    /// Supported drivers: `mariadb` | `mysql` | `postgres` | `sqlite` | `tidb`.
    #[inline]
    fn driver_name() -> &'static str {
        super::DRIVER_NAME
    }

    /// Returns the prefix for the table name.
    #[inline]
    fn table_prefix() -> &'static str {
        *super::TABLE_PREFIX
    }

    /// Returns the prefix for the model namespace.
    #[inline]
    fn namespace_prefix() -> &'static str {
        *super::NAMESPACE_PREFIX
    }

    /// Returns the table name.
    #[inline]
    fn table_name() -> &'static str {
        Self::TABLE_NAME.unwrap_or_else(|| [Self::table_prefix(), Self::MODEL_NAME].concat().leak())
    }

    /// Returns the model namespace.
    #[inline]
    fn model_namespace() -> &'static str {
        [Self::namespace_prefix(), Self::MODEL_NAME].concat().leak()
    }

    /// Returns the primary key as a JSON value.
    #[inline]
    fn primary_key_value(&self) -> JsonValue {
        self.primary_key().to_string().into()
    }

    /// Returns the primary key column.
    #[inline]
    fn primary_key_column() -> &'static Column<'static> {
        Self::get_column(Self::PRIMARY_KEY_NAME)
            .expect("the primary key column should always exist")
    }

    /// Gets a column for the field.
    #[inline]
    fn get_column(key: &str) -> Option<&Column<'static>> {
        let key = if let Some((name, field)) = key.split_once('.') {
            if Self::model_name() == name || Self::table_name() == name {
                field
            } else {
                return None;
            }
        } else {
            key
        };
        Self::columns().iter().find(|col| col.name() == key)
    }

    /// Gets a column for the field if it is writable.
    #[inline]
    fn get_writable_column(key: &str) -> Option<&Column<'static>> {
        let key = if let Some((name, field)) = key.split_once('.') {
            if Self::model_name() == name || Self::table_name() == name {
                field
            } else {
                return None;
            }
        } else {
            key
        };
        Self::columns()
            .iter()
            .find(|col| col.name() == key && !col.is_read_only())
    }

    /// Returns `true` if the model has a column for the specific field.
    #[inline]
    fn has_column(key: &str) -> bool {
        let key = if let Some((name, field)) = key.split_once('.') {
            if Self::model_name() == name || Self::table_name() == name {
                field
            } else {
                return false;
            }
        } else {
            key
        };
        Self::columns().iter().any(|col| col.name() == key)
    }

    /// Constructs a default `Query` for the model.
    #[inline]
    fn default_query() -> Query {
        let mut query = Query::default();
        query.allow_fields(Self::fields());
        query.deny_fields(Self::write_only_fields());
        query
    }

    /// Constructs a default `Mutation` for the model.
    #[inline]
    fn default_mutation() -> Mutation {
        let mut mutation = Mutation::default();
        mutation.allow_fields(Self::fields());
        mutation.deny_fields(Self::read_only_fields());
        mutation
    }

    /// Initializes the model reader.
    #[inline]
    fn init_reader() -> Result<&'static ConnectionPool, Error> {
        GlobalPool::get(Self::READER_NAME)
            .ok_or_else(|| warn!("connection to the database is unavailable"))
    }

    /// Initializes the model writer.
    #[inline]
    fn init_writer() -> Result<&'static ConnectionPool, Error> {
        GlobalPool::get(Self::WRITER_NAME)
            .ok_or_else(|| warn!("connection to the database is unavailable"))
    }

    /// Creates a database table for the model.
    async fn create_table() -> Result<(), Error> {
        if !super::AUTO_MIGRATION.load(Relaxed) {
            return Ok(());
        }
        Self::before_create_table().await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = Self::table_name();
        let table_name_escaped = Query::table_name_escaped::<Self>();
        let columns = Self::columns();
        let mut definitions = columns
            .iter()
            .map(|col| col.field_definition(primary_key_name))
            .collect::<Vec<_>>();
        for col in columns {
            let mut constraints = col.constraints();
            if !constraints.is_empty() {
                definitions.append(&mut constraints);
            }
        }

        let definitions = definitions.join(",\n  ");
        let sql = format!("CREATE TABLE IF NOT EXISTS {table_name_escaped} (\n  {definitions}\n);");
        let pool = Self::init_writer()?.pool();
        if let Err(err) = pool.execute(&sql).await {
            tracing::error!(table_name, "fail to execute `{sql}`");
            return Err(err);
        }
        Self::after_create_table().await?;
        Ok(())
    }

    /// Synchronizes the table schema for the model.
    async fn synchronize_schema() -> Result<(), Error> {
        if !super::AUTO_MIGRATION.load(Relaxed) {
            return Ok(());
        }

        let connection_pool = Self::init_writer()?;
        let model_name = Self::model_name();
        let table_name = Self::table_name();
        let table_name_escaped = Query::table_name_escaped::<Self>();
        let sql = if cfg!(any(
            feature = "orm-mariadb",
            feature = "orm-mysql",
            feature = "orm-tidb"
        )) {
            let table_schema = connection_pool.database();
            format!(
                "SELECT column_name, data_type, column_default, is_nullable \
                    FROM information_schema.columns \
                        WHERE table_schema = '{table_schema}' AND table_name = '{table_name}';"
            )
        } else if cfg!(feature = "orm-postgres") {
            format!(
                "SELECT column_name, data_type, column_default, is_nullable \
                    FROM information_schema.columns \
                        WHERE table_schema = 'public' AND table_name = '{table_name}';"
            )
        } else {
            format!(
                "SELECT p.name AS column_name, p.type AS data_type, \
                        p.dflt_value AS column_default, p.[notnull] AS is_not_null \
                    FROM sqlite_master m LEFT OUTER JOIN pragma_table_info((m.name)) p
                        ON m.name <> p.name WHERE m.name = '{table_name}';"
            )
        };
        let pool = connection_pool.pool();
        let rows = pool.fetch(&sql).await?;
        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(Map::decode_row(&row)?);
        }

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        for col in Self::columns() {
            let column_type = col.column_type();
            let column_name = col
                .extra()
                .get_str("column_name")
                .unwrap_or_else(|| col.name());
            let column_opt = data.iter().find(|d| {
                d.get_str("column_name")
                    .or_else(|| d.get_str("COLUMN_NAME"))
                    == Some(column_name)
            });
            if let Some(d) = column_opt {
                let data_type = d.get_str("data_type").or_else(|| d.get_str("DATA_TYPE"));
                let column_default = d
                    .get_str("column_default")
                    .or_else(|| d.get_str("COLUMN_DEFAULT"));
                let is_not_null = if cfg!(any(feature = "orm-mysql", feature = "orm-postgres")) {
                    d.get_str("is_nullable")
                        .or_else(|| d.get_str("IS_NULLABLE"))
                        .unwrap_or("YES")
                        .eq_ignore_ascii_case("NO")
                } else {
                    d.get_str("is_not_null") == Some("1")
                };
                if !data_type.is_some_and(|t| col.is_compatible(t)) {
                    tracing::warn!(
                        model_name,
                        table_name,
                        column_name,
                        column_type,
                        data_type,
                        column_default,
                        "data type of `{column_name}` should be altered as `{column_type}`",
                    );
                } else if col.is_not_null() != is_not_null && column_name != primary_key_name {
                    tracing::warn!(
                        model_name,
                        table_name,
                        column_name,
                        column_type,
                        data_type,
                        column_default,
                        is_not_null,
                        "`NOT NULL` constraint of `{column_name}` should be consistent",
                    );
                }
            } else {
                let column_definition = col.field_definition(primary_key_name);
                let sql =
                    format!("ALTER TABLE {table_name_escaped} ADD COLUMN {column_definition};");
                pool.execute(&sql).await?;
                tracing::warn!(
                    model_name,
                    table_name,
                    column_name,
                    column_type,
                    "a new column `{column_name}` has been added",
                );
            }
        }
        Ok(())
    }

    /// Creates indexes for the model.
    async fn create_indexes() -> Result<u64, Error> {
        if !super::AUTO_MIGRATION.load(Relaxed) {
            return Ok(0);
        }

        let pool = Self::init_writer()?.pool();
        let columns = Self::columns();
        let table_name = Self::table_name();
        let table_name_escaped = Query::table_name_escaped::<Self>();
        let mut rows = 0;
        if cfg!(any(
            feature = "orm-mariadb",
            feature = "orm-mysql",
            feature = "orm-tidb"
        )) {
            let sql = format!("SHOW INDEXES FROM {table_name_escaped}");
            if pool.fetch(&sql).await?.len() > 1 {
                return Ok(0);
            }

            let mut text_search_columns = Vec::new();
            for col in columns {
                if let Some(index_type) = col.index_type() {
                    let column_name = col.name();
                    if matches!(index_type, "fulltext" | "text") {
                        text_search_columns.push(column_name);
                    } else if matches!(index_type, "unique" | "spatial") {
                        let index_type = index_type.to_uppercase();
                        let sql = format!(
                            "CREATE {index_type} INDEX {table_name}_{column_name}_index \
                                ON {table_name_escaped} ({column_name});"
                        );
                        rows = pool.execute(&sql).await?.rows_affected().max(rows);
                    } else if matches!(index_type, "btree" | "hash") {
                        let index_type = index_type.to_uppercase();
                        let sql = format!(
                            "CREATE INDEX {table_name}_{column_name}_index \
                                ON {table_name_escaped} ({column_name}) USING {index_type};"
                        );
                        rows = pool.execute(&sql).await?.rows_affected().max(rows);
                    }
                }
            }
            if !text_search_columns.is_empty() {
                let text_search_columns = text_search_columns.join(", ");
                let sql = format!(
                    "CREATE FULLTEXT INDEX {table_name}_text_search_index \
                        ON {table_name_escaped} ({text_search_columns});"
                );
                rows = pool.execute(&sql).await?.rows_affected().max(rows);
            }
        } else if cfg!(feature = "orm-postgres") {
            let mut text_search_columns = Vec::new();
            let mut text_search_languages = Vec::new();
            for col in columns {
                if let Some(index_type) = col.index_type() {
                    let column_name = col.name();
                    if index_type.starts_with("text") {
                        let language = index_type.strip_prefix("text:").unwrap_or("english");
                        let column = format!("coalesce({column_name}, '')");
                        text_search_languages.push(language);
                        text_search_columns.push((language, column));
                    } else if index_type == "unique" {
                        let sql = format!(
                            "CREATE UNIQUE INDEX IF NOT EXISTS {table_name}_{column_name}_index \
                                ON {table_name_escaped} ({column_name});"
                        );
                        rows = pool.execute(&sql).await?.rows_affected().max(rows);
                    } else {
                        let sort_order = if index_type == "btree" { " DESC" } else { "" };
                        let sql = format!(
                            "CREATE INDEX IF NOT EXISTS {table_name}_{column_name}_index \
                                ON {table_name_escaped} \
                                    USING {index_type}({column_name}{sort_order});"
                        );
                        rows = pool.execute(&sql).await?.rows_affected().max(rows);
                    }
                }
            }
            for language in text_search_languages {
                let text = text_search_columns
                    .iter()
                    .filter_map(|col| (col.0 == language).then_some(col.1.as_str()))
                    .collect::<Vec<_>>()
                    .join(" || ' ' || ");
                let text_search = format!("to_tsvector('{language}', {text})");
                let sql = format!(
                    "CREATE INDEX IF NOT EXISTS {table_name}_text_search_{language}_index \
                        ON {table_name_escaped} USING gin({text_search});"
                );
                rows = pool.execute(&sql).await?.rows_affected().max(rows);
            }
        } else {
            for col in columns {
                if let Some(index_type) = col.index_type() {
                    let column_name = col.name();
                    let index_type = if index_type == "unique" { "UNIQUE" } else { "" };
                    let sql = format!(
                        "CREATE {index_type} INDEX IF NOT EXISTS {table_name}_{column_name}_index \
                            ON {table_name_escaped} ({column_name});"
                    );
                    rows = pool.execute(&sql).await?.rows_affected().max(rows);
                }
            }
        }
        Ok(rows)
    }

    /// Prepares the SQL to insert the model into the table.
    async fn prepare_insert(self) -> Result<QueryContext, Error> {
        let map = self.into_map();
        let table_name = Query::table_name_escaped::<Self>();
        let columns = Self::columns();

        let mut fields = Vec::with_capacity(columns.len());
        let values = columns
            .iter()
            .filter_map(|col| {
                if col.auto_increment() {
                    None
                } else {
                    let name = col.name();
                    fields.push(name);
                    Some(col.encode_value(map.get(name)))
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let fields = fields.join(", ");
        let sql = format!("INSERT INTO {table_name} ({fields}) VALUES ({values});");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Inserts the model into the table.
    async fn insert(mut self) -> Result<QueryContext, Error> {
        let model_data = self.before_insert().await?;
        let mut ctx = self.prepare_insert().await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        let (last_insert_id, rows_affected) = Query::parse_query_result(query_result);
        let success = rows_affected == 1;
        if let Some(last_insert_id) = last_insert_id {
            ctx.set_last_insert_id(last_insert_id);
        }
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        Self::after_insert(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect 1 row",
                rows_affected
            );
        }
    }

    /// Prepares the SQL to insert many models into the table.
    async fn prepare_insert_many(models: Vec<Self>) -> Result<QueryContext, Error> {
        if models.is_empty() {
            bail!("the list of models to be inserted should be nonempty");
        }

        let columns = Self::columns();
        let mut values = Vec::with_capacity(models.len());
        for mut model in models.into_iter() {
            let _model_data = model.before_insert().await?;

            let map = model.into_map();
            let entries = columns
                .iter()
                .map(|col| col.encode_value(map.get(col.name())))
                .collect::<Vec<_>>()
                .join(", ");
            values.push(format!("({entries})"));
        }

        let table_name = Query::table_name_escaped::<Self>();
        let fields = Self::fields().join(", ");
        let values = values.join(", ");
        let sql = format!("INSERT INTO {table_name} ({fields}) VALUES {values};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Inserts many models into the table.
    async fn insert_many(models: Vec<Self>) -> Result<QueryContext, Error> {
        let mut ctx = Self::prepare_insert_many(models).await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        ctx.set_query_result(query_result.rows_affected(), true);
        Self::after_scan(&ctx).await?;
        Ok(ctx)
    }

    /// Prepares the SQL to update the model in the table.
    async fn prepare_update(self) -> Result<QueryContext, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = Query::table_name_escaped::<Self>();
        let primary_key = Query::escape_string(self.primary_key());
        let map = self.into_map();
        let read_only_fields = Self::read_only_fields();
        let num_writable_fields = Self::fields().len() - read_only_fields.len();
        let mut mutations = Vec::with_capacity(num_writable_fields);
        for col in Self::columns() {
            let field = col.name();
            if !read_only_fields.contains(&field) {
                let value = col.encode_value(map.get(field));
                let field = Query::format_field(field);
                mutations.push(format!("{field} = {value}"));
            }
        }

        let mutations = mutations.join(", ");
        let sql = format!(
            "UPDATE {table_name} SET {mutations} WHERE {primary_key_name} = {primary_key};"
        );
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Updates the model in the table.
    async fn update(mut self) -> Result<QueryContext, Error> {
        let model_data = self.before_update().await?;
        let mut ctx = self.prepare_update().await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        Self::after_update(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect 1 row",
                rows_affected
            );
        }
    }

    /// Prepares the SQL to update at most one model selected by the query in the table.
    async fn prepare_update_one(
        query: &Query,
        mutation: &mut Mutation,
    ) -> Result<QueryContext, Error> {
        Self::before_mutation(query, mutation).await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let updates = mutation.format_updates::<Self>();
        let sql = if cfg!(any(
            feature = "orm-mariadb",
            feature = "orm-mysql",
            feature = "orm-tidb"
        )) {
            // MySQL doesn't yet support 'LIMIT & IN/ALL/ANY/SOME subquery'
            // and self-referencing in UPDATE/DELETE
            format!(
                "UPDATE {table_name} SET {updates} WHERE {primary_key_name} IN \
                    (SELECT * from (SELECT {primary_key_name} FROM {table_name} {filters}) AS t);"
            )
        } else {
            // Both PostgreQL and SQLite support a `LIMIT` in subquery
            let sort = query.format_sort();
            format!(
                "UPDATE {table_name} SET {updates} WHERE {primary_key_name} IN \
                    (SELECT {primary_key_name} FROM {table_name} {filters} {sort} LIMIT 1);"
            )
        };
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Updates at most one model selected by the query in the table.
    async fn update_one(query: &Query, mutation: &mut Mutation) -> Result<QueryContext, Error> {
        let mut ctx = Self::prepare_update_one(query, mutation).await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected <= 1;
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        Self::after_mutation(&ctx).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect at most 1 row",
                rows_affected
            );
        }
    }

    /// Prepares the SQL to update many models selected by the query in the table.
    async fn prepare_update_many(
        query: &Query,
        mutation: &mut Mutation,
    ) -> Result<QueryContext, Error> {
        Self::before_mutation(query, mutation).await?;

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let updates = mutation.format_updates::<Self>();
        let sql = format!("UPDATE {table_name} SET {updates} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Updates many models selected by the query in the table.
    async fn update_many(query: &Query, mutation: &mut Mutation) -> Result<QueryContext, Error> {
        let mut ctx = Self::prepare_update_many(query, mutation).await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        ctx.set_query_result(query_result.rows_affected(), true);
        Self::after_scan(&ctx).await?;
        Self::after_mutation(&ctx).await?;
        Ok(ctx)
    }

    /// Prepares the SQL to update or insert the model into the table.
    async fn prepare_upsert(self) -> Result<QueryContext, Error> {
        let map = self.into_map();
        let table_name = Query::table_name_escaped::<Self>();
        let fields = Self::fields();
        let num_fields = fields.len();
        let read_only_fields = Self::read_only_fields();
        let num_writable_fields = num_fields - read_only_fields.len();
        let mut values = Vec::with_capacity(num_fields);
        let mut mutations = Vec::with_capacity(num_writable_fields);
        for col in Self::columns() {
            let field = col.name();
            let value = col.encode_value(map.get(field));
            if !read_only_fields.contains(&field) {
                let field = Query::format_field(field);
                mutations.push(format!("{field} = {value}"));
            }
            values.push(value);
        }

        let fields = fields.join(", ");
        let values = values.join(", ");
        let mutations = mutations.join(", ");
        let sql = if cfg!(any(
            feature = "orm-mariadb",
            feature = "orm-mysql",
            feature = "orm-tidb"
        )) {
            format!(
                "INSERT INTO {table_name} ({fields}) VALUES ({values}) \
                    ON DUPLICATE KEY UPDATE {mutations};"
            )
        } else {
            let primary_key_name = Self::PRIMARY_KEY_NAME;

            // Both PostgreQL and SQLite (3.24+) support this syntax.
            format!(
                "INSERT INTO {table_name} ({fields}) VALUES ({values}) \
                    ON CONFLICT ({primary_key_name}) DO UPDATE SET {mutations};"
            )
        };
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Updates or inserts the model into the table.
    async fn upsert(mut self) -> Result<QueryContext, Error> {
        let model_data = self.before_upsert().await?;
        let mut ctx = self.prepare_upsert().await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        let (last_insert_id, rows_affected) = Query::parse_query_result(query_result);
        let success = rows_affected == 1;
        if let Some(last_insert_id) = last_insert_id {
            ctx.set_last_insert_id(last_insert_id);
        }
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        Self::after_upsert(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect 1 row",
                rows_affected
            );
        }
    }

    /// Prepares the SQL to delete the model in the table.
    async fn prepare_delete() -> Result<QueryContext, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = Query::table_name_escaped::<Self>();
        let placeholder = Query::placeholder(1);
        let sql = if cfg!(feature = "orm-postgres") {
            let type_annotation = Self::primary_key_column().type_annotation();
            format!(
                "DELETE FROM {table_name} \
                    WHERE {primary_key_name} = ({placeholder}){type_annotation};"
            )
        } else {
            format!("DELETE FROM {table_name} WHERE {primary_key_name} = {placeholder};")
        };
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Deletes the model in the table.
    async fn delete(mut self) -> Result<QueryContext, Error> {
        let model_data = self.before_delete().await?;
        let mut ctx = Self::prepare_delete().await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let primary_key = self.primary_key();
        let query_result = pool.execute_with(ctx.query(), &[primary_key]).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.add_argument(primary_key);
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        self.after_delete(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect 1 row",
                rows_affected
            );
        }
    }

    /// Prepares the SQL to delete at most one model selected by the query in the table.
    async fn prepare_delete_one(query: &Query) -> Result<QueryContext, Error> {
        Self::before_query(query).await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let sql = format!(
            "DELETE FROM {table_name} WHERE {primary_key_name} IN \
                (SELECT {primary_key_name} FROM {table_name} {filters} {sort} LIMIT 1);"
        );
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Deletes at most one model selected by the query in the table.
    async fn delete_one(query: &Query) -> Result<QueryContext, Error> {
        let mut ctx = Self::prepare_delete_one(query).await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected <= 1;
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect at most 1 row",
                rows_affected
            );
        }
    }

    /// Prepares the SQL to delete many models selected by the query in the table.
    async fn prepare_delete_many(query: &Query) -> Result<QueryContext, Error> {
        Self::before_query(query).await?;

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let sql = format!("DELETE FROM {table_name} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Deletes many models selected by the query in the table.
    async fn delete_many(query: &Query) -> Result<QueryContext, Error> {
        let mut ctx = Self::prepare_delete_many(query).await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute(ctx.query()).await?;
        ctx.set_query_result(query_result.rows_affected(), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(ctx)
    }

    /// Finds a list of models selected by the query in the table,
    /// and decodes it as `Vec<T>`.
    async fn find<T>(query: &Query) -> Result<Vec<T>, Error>
    where
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        Self::before_query(query).await?;

        let table_name = query.format_table_name::<Self>();
        let projection = query.format_table_fields::<Self>();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} {pagination};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(&sql);

        let pool = Self::acquire_reader().await?.pool();
        let rows = pool.fetch(ctx.query()).await?;
        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(T::decode_row(&row)?);
        }
        ctx.set_query_result(u64::try_from(data.len())?, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Finds a list of models selected by the query in the table,
    /// and parses it as `Vec<T>`.
    async fn find_as<T: DeserializeOwned>(query: &Query) -> Result<Vec<T>, Error> {
        let mut data = Self::find::<Map>(query).await?;
        let translate_enabled = query.translate_enabled();
        for model in data.iter_mut() {
            Self::after_decode(model).await?;
            translate_enabled.then(|| Self::translate_model(model));
        }
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    /// Finds one model selected by the query in the table,
    /// and decodes it as an instance of type `T`.
    async fn find_one<T>(query: &Query) -> Result<Option<T>, Error>
    where
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        Self::before_query(query).await?;

        let table_name = query.format_table_name::<Self>();
        let projection = query.format_table_fields::<Self>();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} LIMIT 1;");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let pool = Self::acquire_reader().await?.pool();
        let (num_rows, data) = if let Some(row) = pool.fetch_optional(ctx.query()).await? {
            (1, Some(T::decode_row(&row)?))
        } else {
            (0, None)
        };
        ctx.set_query_result(num_rows, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Finds one model selected by the query in the table,
    /// and parses it as an instance of type `T`.
    async fn find_one_as<T: DeserializeOwned>(query: &Query) -> Result<Option<T>, Error> {
        match Self::find_one::<Map>(query).await? {
            Some(mut data) => {
                Self::after_decode(&mut data).await?;
                query
                    .translate_enabled()
                    .then(|| Self::translate_model(&mut data));
                serde_json::from_value(data.into()).map_err(Error::from)
            }
            None => Ok(None),
        }
    }

    /// Populates the related data in the corresponding `columns` for `Vec<Map>` using
    /// a merged select on the primary key, which solves the `N+1` problem.
    async fn populate(
        query: &mut Query,
        data: &mut Vec<Map>,
        columns: &[&str],
    ) -> Result<u64, Error> {
        Self::before_query(query).await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut values = Vec::new();
        for row in data.iter() {
            for &col in columns {
                if let Some(value) = row.get(col) {
                    if let JsonValue::Array(vec) = value {
                        for value in vec {
                            if !values.contains(value) {
                                values.push(value.clone());
                            }
                        }
                    } else if !values.contains(value) {
                        values.push(value.clone());
                    }
                }
            }
        }

        let num_values = values.len();
        if num_values > 0 {
            let primary_key_values = Map::from_entry("$in", values);
            query.add_filter(primary_key_name, primary_key_values);
        } else {
            return Ok(0);
        }

        let table_name = query.format_table_name::<Self>();
        let projection = query.format_table_fields::<Self>();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT {projection} FROM {table_name} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(&sql);

        let pool = Self::acquire_reader().await?.pool();
        let rows = pool.fetch(ctx.query()).await?;
        let translate_enabled = query.translate_enabled();
        let mut associations = Vec::with_capacity(num_values);
        for row in rows {
            let mut map = Map::decode_row(&row)?;
            let primary_key = map.get(primary_key_name).cloned();
            Self::after_decode(&mut map).await?;
            translate_enabled.then(|| Self::translate_model(&mut map));
            if let Some(key) = primary_key {
                associations.push((key, map));
            }
        }

        let associations_len = u64::try_from(associations.len())?;
        ctx.set_query_result(associations_len, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;

        for row in data {
            for &col in columns {
                if let Some(vec) = row.get_array(col).filter(|vec| !vec.is_empty()) {
                    let populated_field = [col, "_populated"].concat();
                    let populated_values = vec
                        .iter()
                        .map(|key| {
                            let populated_value = associations
                                .iter()
                                .find_map(|(k, v)| (key == k).then_some(v));
                            if let Some(value) = populated_value {
                                value.clone().into()
                            } else {
                                key.clone()
                            }
                        })
                        .collect::<Vec<_>>();
                    row.upsert(populated_field, populated_values);
                } else if let Some(key) = row.get(col) {
                    let populated_value = associations
                        .iter()
                        .find_map(|(k, v)| (key == k).then_some(v));
                    if let Some(value) = populated_value {
                        let populated_field = [col, "_populated"].concat();
                        row.upsert(populated_field, value.clone());
                    }
                }
            }
        }
        Ok(associations_len)
    }

    /// Populates the related data in the corresponding `columns` for `Map` using
    /// a merged select on the primary key, which solves the `N+1` problem.
    async fn populate_one(
        query: &mut Query,
        data: &mut Map,
        columns: &[&str],
    ) -> Result<(), Error> {
        Self::before_query(query).await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut values = Vec::new();
        for &col in columns {
            if let Some(value) = data.get(col) {
                if let JsonValue::Array(vec) = value {
                    for value in vec {
                        if !values.contains(value) {
                            values.push(value.clone());
                        }
                    }
                } else if !values.contains(value) {
                    values.push(value.clone());
                }
            }
        }

        let num_values = values.len();
        if num_values > 0 {
            let primary_key_values = Map::from_entry("$in", values);
            query.add_filter(primary_key_name, primary_key_values);
        } else {
            return Ok(());
        }

        let table_name = query.format_table_name::<Self>();
        let projection = query.format_projection();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT {projection} FROM {table_name} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(&sql);

        let pool = Self::acquire_reader().await?.pool();
        let rows = pool.fetch(ctx.query()).await?;
        let translate_enabled = query.translate_enabled();
        let mut associations = Vec::with_capacity(num_values);
        for row in rows {
            let mut map = Map::decode_row(&row)?;
            let primary_key = map.get(primary_key_name).cloned();
            Self::after_decode(&mut map).await?;
            translate_enabled.then(|| Self::translate_model(&mut map));
            if let Some(key) = primary_key {
                associations.push((key, map));
            }
        }
        ctx.set_query_result(u64::try_from(associations.len())?, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;

        for &col in columns {
            if let Some(vec) = data.get_array(col).filter(|vec| !vec.is_empty()) {
                let populated_field = [col, "_populated"].concat();
                let populated_values = vec
                    .iter()
                    .map(|key| {
                        let populated_value = associations
                            .iter()
                            .find_map(|(k, v)| (key == k).then_some(v));
                        if let Some(value) = populated_value {
                            value.clone().into()
                        } else {
                            key.clone()
                        }
                    })
                    .collect::<Vec<_>>();
                data.upsert(populated_field, populated_values);
            } else if let Some(key) = data.get(col) {
                let populated_value = associations
                    .iter()
                    .find_map(|(k, v)| (key == k).then_some(v));
                if let Some(value) = populated_value {
                    let populated_field = [col, "_populated"].concat();
                    data.upsert(populated_field, value.clone());
                }
            }
        }
        Ok(())
    }

    /// Performs a left outer join to another table to filter rows in the joined table,
    /// and decodes it as `Vec<T>`.
    async fn lookup<M, T>(query: &Query, columns: &[(&str, &str)]) -> Result<Vec<T>, Error>
    where
        M: Schema,
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        Self::before_query(query).await?;

        let model_name = Self::model_name();
        let other_model_name = M::model_name();
        let table_name = query.format_table_name::<Self>();
        let other_table_name = query.format_table_name::<M>();
        let projection = query.format_table_fields::<Self>();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let on_expressions = columns
            .iter()
            .map(|(left_col, right_col)| {
                let left_col = format!("{model_name}.{left_col}");
                let right_col = format!("{other_model_name}.{right_col}");
                let left_col_field = Query::format_field(&left_col);
                let right_col_field = Query::format_field(&right_col);
                format!("{left_col_field} = {right_col_field}")
            })
            .collect::<Vec<_>>()
            .join(" AND ");
        let sql = format!(
            "SELECT {projection} FROM {table_name} \
                LEFT OUTER JOIN {other_table_name} \
                    ON {on_expressions} {filters} {sort} {pagination};"
        );
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(&sql);

        let pool = Self::acquire_reader().await?.pool();
        let rows = pool.fetch(ctx.query()).await?;
        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(T::decode_row(&row)?);
        }
        ctx.set_query_result(u64::try_from(data.len())?, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Performs a left outer join to another table to filter rows in the "joined" table,
    /// and parses it as `Vec<T>`.
    async fn lookup_as<M, T>(query: &Query, columns: &[(&str, &str)]) -> Result<Vec<T>, Error>
    where
        M: Schema,
        T: DeserializeOwned,
    {
        let mut data = Self::lookup::<M, Map>(query, columns).await?;
        let translate_enabled = query.translate_enabled();
        for model in data.iter_mut() {
            Self::after_decode(model).await?;
            translate_enabled.then(|| Self::translate_model(model));
        }
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    /// Checks whether there is a model selected by the query in the table.
    async fn exists(query: &Query) -> Result<bool, Error> {
        Self::before_query(query).await?;

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT 1 FROM {table_name} {filters} LIMIT 1;");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let pool = Self::acquire_reader().await?.pool();
        let optional_row = pool.fetch_optional(ctx.query()).await?;
        let num_rows = if optional_row.is_some() { 1 } else { 0 };
        ctx.set_query_result(num_rows, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(num_rows == 1)
    }

    /// Counts the number of rows selected by the query in the table.
    async fn count(query: &Query) -> Result<u64, Error> {
        Self::before_count(query).await?;

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT count(*) AS count FROM {table_name} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let pool = Self::acquire_reader().await?.pool();
        let row = pool.fetch_one(ctx.query()).await?;
        let map = Map::decode_row(&row)?;

        // SQLite may return a string value for the count value.
        let count = map.parse_u64("count").transpose()?.unwrap_or_default();
        ctx.set_query_result(count, true);
        Self::after_scan(&ctx).await?;
        Self::after_count(&ctx).await?;
        Ok(count)
    }

    /// Counts the number of rows selected by the query in the table.
    /// The boolean value determines whether it only counts distinct values or not.
    async fn count_many<T>(query: &Query, columns: &[(&str, bool)]) -> Result<T, Error>
    where
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        Self::before_count(query).await?;

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let projection = columns
            .iter()
            .map(|&(key, distinct)| {
                let field = Query::format_field(key);
                if key != "*" {
                    if distinct {
                        format!(r#"count(distinct {field}) as {key}_count_distinct"#)
                    } else {
                        format!(r#"count({field}) as {key}_count"#)
                    }
                } else {
                    "count(*)".to_owned()
                }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("SELECT {projection} FROM {table_name} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let pool = Self::acquire_reader().await?.pool();
        let row = pool.fetch_one(ctx.query()).await?;
        ctx.set_query_result(1, true);
        Self::after_scan(&ctx).await?;
        Self::after_count(&ctx).await?;
        T::decode_row(&row).map_err(Error::from)
    }

    /// Counts the number of rows selected by the query in the table,
    /// and parses it as an instance of type `T`.
    async fn count_many_as<T: DeserializeOwned>(
        query: &Query,
        columns: &[(&str, bool)],
    ) -> Result<T, Error> {
        let map = Self::count_many::<Map>(query, columns).await?;
        serde_json::from_value(map.into()).map_err(Error::from)
    }

    /// Executes the query in the table, and returns the total number of rows affected.
    async fn execute(query: &str, params: Option<&Map>) -> Result<QueryContext, Error> {
        let (sql, values) = Query::prepare_query(query, params);
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let mut arguments = values
            .iter()
            .map(|v| v.to_string_unquoted())
            .collect::<Vec<_>>();
        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute_with(ctx.query(), &arguments).await?;
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(query_result.rows_affected(), true);
        Self::after_scan(&ctx).await?;
        Ok(ctx)
    }

    /// Executes the query in the table, and decodes it as `Vec<T>`.
    async fn query<T>(query: &str, params: Option<&Map>) -> Result<Vec<T>, Error>
    where
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        let (sql, values) = Query::prepare_query(query, params);
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let mut arguments = values
            .iter()
            .map(|v| v.to_string_unquoted())
            .collect::<Vec<_>>();
        let pool = Self::acquire_reader().await?.pool();
        let rows = pool.fetch_with(ctx.query(), &arguments).await?;
        let mut data = Vec::with_capacity(rows.len());
        for row in rows {
            data.push(T::decode_row(&row)?);
        }
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(u64::try_from(data.len())?, true);
        Self::after_scan(&ctx).await?;
        Ok(data)
    }

    /// Executes the query in the table, and parses it as `Vec<T>`.
    async fn query_as<T: DeserializeOwned>(
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let mut data = Self::query::<Map>(query, params).await?;
        for model in data.iter_mut() {
            Self::after_decode(model).await?;
        }
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    /// Executes the query in the table, and decodes it as an instance of type `T`.
    async fn query_one<T>(query: &str, params: Option<&Map>) -> Result<Option<T>, Error>
    where
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        let (sql, values) = Query::prepare_query(query, params);
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let mut arguments = values
            .iter()
            .map(|v| v.to_string_unquoted())
            .collect::<Vec<_>>();
        let pool = Self::acquire_reader().await?.pool();
        let optional_row = pool.fetch_optional_with(ctx.query(), &arguments).await?;
        let (num_rows, data) = if let Some(row) = optional_row {
            (1, Some(T::decode_row(&row)?))
        } else {
            (0, None)
        };
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(num_rows, true);
        Self::after_scan(&ctx).await?;
        Ok(data)
    }

    /// Executes the query in the table, and parses it as an instance of type `T`.
    async fn query_one_as<T: DeserializeOwned>(
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        match Self::query_one::<Map>(query, params).await? {
            Some(mut data) => {
                Self::after_decode(&mut data).await?;
                serde_json::from_value(data.into()).map_err(Error::from)
            }
            None => Ok(None),
        }
    }

    /// Prepares the SQL to delete a model selected by the primary key in the table.
    async fn prepare_delete_by_id() -> Result<QueryContext, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = Query::table_name_escaped::<Self>();
        let placeholder = Query::placeholder(1);
        let sql = if cfg!(feature = "orm-postgres") {
            let type_annotation = Self::primary_key_column().type_annotation();
            format!(
                "DELETE FROM {table_name} \
                    WHERE {primary_key_name} = ({placeholder}){type_annotation};"
            )
        } else {
            format!("DELETE FROM {table_name} WHERE {primary_key_name} = {placeholder};")
        };
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        if cfg!(debug_assertions) && super::DEBUG_ONLY.load(Relaxed) {
            ctx.cancel();
        }
        Ok(ctx)
    }

    /// Deletes a model selected by the primary key in the table.
    async fn delete_by_id(primary_key: &Self::PrimaryKey) -> Result<QueryContext, Error> {
        let mut ctx = Self::prepare_delete_by_id().await?;
        if ctx.is_cancelled() {
            return Ok(ctx);
        }

        let pool = Self::acquire_writer().await?.pool();
        let query_result = pool.execute_with(ctx.query(), &[primary_key]).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.add_argument(primary_key);
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        if success {
            Ok(ctx)
        } else {
            bail!(
                "{} rows are affected while it is expected to affect 1 row",
                rows_affected
            );
        }
    }

    /// Finds a model selected by the primary key in the table,
    /// and decodes it as an instance of type `T`.
    async fn find_by_id<T>(primary_key: &Self::PrimaryKey) -> Result<Option<T>, Error>
    where
        T: DecodeRow<DatabaseRow, Error = Error>,
    {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let query = Self::default_query();
        let table_name = query.format_table_name::<Self>();
        let projection = query.format_projection();
        let placeholder = Query::placeholder(1);
        let sql = if cfg!(feature = "orm-postgres") {
            let type_annotation = Self::primary_key_column().type_annotation();
            format!(
                "SELECT {projection} FROM {table_name} \
                    WHERE {primary_key_name} = ({placeholder}){type_annotation};"
            )
        } else {
            format!(
                "SELECT {projection} FROM {table_name} WHERE {primary_key_name} = {placeholder};"
            )
        };
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let pool = Self::acquire_reader().await?.pool();
        let optional_row = pool
            .fetch_optional_with(ctx.query(), &[primary_key])
            .await?;
        let (num_rows, data) = if let Some(row) = optional_row {
            (1, Some(T::decode_row(&row)?))
        } else {
            (0, None)
        };
        ctx.add_argument(primary_key);
        ctx.set_query_result(num_rows, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Finds a model selected by the primary key in the table, and parses it as `Self`.
    async fn try_get_model(primary_key: &Self::PrimaryKey) -> Result<Self, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let query = Self::default_query();
        let table_name = query.format_table_name::<Self>();
        let projection = query.format_projection();
        let placeholder = Query::placeholder(1);
        let sql = if cfg!(feature = "orm-postgres") {
            let type_annotation = Self::primary_key_column().type_annotation();
            format!(
                "SELECT {projection} FROM {table_name} \
                    WHERE {primary_key_name} = ({placeholder}){type_annotation};"
            )
        } else {
            format!(
                "SELECT {projection} FROM {table_name} WHERE {primary_key_name} = {placeholder};"
            )
        };
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);
        ctx.add_argument(primary_key);

        let pool = Self::acquire_reader().await?.pool();
        let optional_row = pool
            .fetch_optional_with(ctx.query(), &[primary_key])
            .await?;
        if let Some(row) = optional_row {
            ctx.set_query_result(1, true);
            Self::after_scan(&ctx).await?;
            Self::after_query(&ctx).await?;

            let mut map = Map::decode_row(&row)?;
            Self::after_decode(&mut map).await?;
            Self::try_from_map(map).map_err(|err| {
                warn!(
                    "fail to decode the value as a model `{}`: {}",
                    Self::MODEL_NAME,
                    err
                )
            })
        } else {
            ctx.set_query_result(0, true);
            Self::after_scan(&ctx).await?;
            Self::after_query(&ctx).await?;
            bail!(
                "404 Not Found: no rows for the model `{}` with the key `{}`",
                Self::MODEL_NAME,
                primary_key
            );
        }
    }

    /// Randomly selects the specified number of models from the table
    /// and returns a list of the primary key values.
    async fn sample(size: usize) -> Result<Vec<JsonValue>, Error> {
        if size == 0 {
            return Ok(Vec::new());
        }

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut query = Query::default();
        query.allow_fields(&[primary_key_name]);
        query.add_filter("$rand", 0.05);
        query.set_limit(size);

        let mut data = Self::find::<Map>(&query)
            .await?
            .into_iter()
            .filter_map(|mut map| map.remove(primary_key_name))
            .collect::<Vec<_>>();
        let remainder_size = size - data.len();
        if remainder_size > 0 {
            let mut query = Query::default();
            query.add_filter(primary_key_name, Map::from_entry("$nin", data.clone()));
            query.allow_fields(&[primary_key_name]);
            query.set_limit(remainder_size);

            let remainder_data = Self::find::<Map>(&query).await?;
            for mut map in remainder_data {
                if let Some(value) = map.remove(primary_key_name) {
                    data.push(value);
                }
            }
        }
        Ok(data)
    }

    /// Filters the values of the primary key.
    async fn filter<T: Into<JsonValue>>(
        primary_key_values: Vec<T>,
    ) -> Result<Vec<JsonValue>, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let limit = primary_key_values.len();
        let mut query = Query::default();
        query.allow_fields(&[primary_key_name]);
        query.add_filter(primary_key_name, Map::from_entry("$in", primary_key_values));
        query.set_limit(limit);

        let data = Self::find::<Map>(&query).await?;
        let mut primary_key_values = Vec::with_capacity(data.len());
        for map in data.into_iter() {
            for (_key, value) in map.into_iter() {
                primary_key_values.push(value);
            }
        }
        Ok(primary_key_values)
    }

    /// Returns `true` if the model is unique on the column values.
    async fn is_unique_on(&self, columns: Vec<(&str, JsonValue)>) -> Result<bool, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut query = Query::default();
        let mut fields = vec![primary_key_name];
        for (field, value) in columns.into_iter() {
            fields.push(field);
            query.add_filter(field, value);
        }
        query.allow_fields(&fields);
        query.set_limit(2);

        let data = Self::find::<Map>(&query).await?;
        match data.len() {
            0 => Ok(true),
            1 => {
                if let Some(value) = data.first().and_then(|m| m.get(primary_key_name)) {
                    Ok(&self.primary_key_value() == value)
                } else {
                    Ok(true)
                }
            }
            _ => Ok(false),
        }
    }
}
