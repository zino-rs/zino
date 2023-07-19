use super::{
    mutation::MutationExt, query::QueryExt, ConnectionPool, DatabaseDriver, DatabaseRow,
    ModelHelper,
};
use crate::{
    error::Error,
    extension::JsonObjectExt,
    model::{Column, DecodeRow, EncodeColumn, ModelHooks, Mutation, Query, QueryContext},
    BoxFuture, JsonValue, Map, Uuid,
};
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use sqlx::{Decode, Row, Transaction, Type};
use std::{fmt::Display, sync::atomic::Ordering::Relaxed};

/// Database schema.
pub trait Schema: 'static + Send + Sync + ModelHooks {
    /// Primary key.
    type PrimaryKey: Default + Display + PartialEq = Uuid;

    /// Model name.
    const MODEL_NAME: &'static str;
    /// Primary key name.
    const PRIMARY_KEY_NAME: &'static str = "id";
    /// Reader name.
    const READER_NAME: &'static str = "main";
    /// Writer name.
    const WRITER_NAME: &'static str = "main";

    /// Returns the primary key value.
    fn primary_key(&self) -> &Self::PrimaryKey;

    /// Returns a reference to the [Avro schema](apache_avro::schema::Schema).
    fn schema() -> &'static apache_avro::Schema;

    /// Returns a reference to the columns.
    fn columns() -> &'static [Column<'static>];

    /// Returns a reference to the column fields.
    fn fields() -> &'static [&'static str];

    /// Returns a reference to the readonly column fields.
    fn readonly_fields() -> &'static [&'static str];

    /// Returns a reference to the writeonly column fields.
    fn writeonly_fields() -> &'static [&'static str];

    /// Retrieves a connection pool for the model reader.
    async fn acquire_reader() -> Result<&'static ConnectionPool, Error>;

    /// Retrieves a connection pool for the model writer.
    async fn acquire_writer() -> Result<&'static ConnectionPool, Error>;

    /// Returns the model name.
    #[inline]
    fn model_name() -> &'static str {
        Self::MODEL_NAME
    }

    /// Returns the model namespace.
    #[inline]
    fn model_namespace() -> &'static str {
        [*super::NAMESPACE_PREFIX, Self::MODEL_NAME]
            .join(":")
            .leak()
    }

    /// Returns the table name.
    #[inline]
    fn table_name() -> &'static str {
        [*super::NAMESPACE_PREFIX, Self::MODEL_NAME]
            .join("_")
            .leak()
    }

    /// Constructs a default `Query` for the model.
    #[inline]
    fn default_query() -> Query {
        let mut query = Query::default();
        query.allow_fields(Self::fields());
        query.deny_fields(Self::writeonly_fields());
        query.set_limit(super::MAX_ROWS.load(Relaxed));
        query
    }

    /// Constructs a default `Mutation` for the model.
    #[inline]
    fn default_mutation() -> Mutation {
        let mut mutation = Mutation::default();
        mutation.allow_fields(Self::fields());
        mutation.deny_fields(Self::readonly_fields());
        mutation
    }

    /// Gets a column for the field.
    #[inline]
    fn get_column(key: &str) -> Option<&Column<'static>> {
        let key = if let Some((name, field)) = key.split_once('.') &&
            (Self::model_name() == name || Self::table_name() == name)
        {
            field
        } else {
            key
        };
        Self::columns().iter().find(|col| col.name() == key)
    }

    /// Initializes the model reader.
    #[inline]
    fn init_reader() -> Result<&'static ConnectionPool, Error> {
        super::SHARED_CONNECTION_POOLS
            .get_pool(Self::READER_NAME)
            .ok_or_else(|| Error::new("connection to the database is unavailable"))
    }

    /// Initializes the model writer.
    #[inline]
    fn init_writer() -> Result<&'static ConnectionPool, Error> {
        super::SHARED_CONNECTION_POOLS
            .get_pool(Self::WRITER_NAME)
            .ok_or_else(|| Error::new("connection to the database is unavailable"))
    }

    /// Creates table for the model.
    async fn create_table() -> Result<(), Error> {
        let pool = Self::init_writer()?.pool();
        Self::before_create_table().await?;

        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let columns = Self::columns()
            .iter()
            .map(|col| {
                let column_name = col.name();
                let column_type = col.column_type();
                let mut column = format!("{column_name} {column_type}");
                if column_name == primary_key_name {
                    column += " PRIMARY KEY";
                } else if let Some(value) = col.default_value() {
                    if cfg!(feature = "orm-mysql") && col.auto_increment() {
                        column += " AUTO_INCREMENT";
                    } else {
                        column = column + " DEFAULT " + &col.format_value(value);
                    }
                } else if col.is_not_null() {
                    column += " NOT NULL";
                }
                column
            })
            .collect::<Vec<_>>()
            .join(",\n  ");
        let sql = format!("CREATE TABLE IF NOT EXISTS {table_name} (\n  {columns}\n);");
        sqlx::query(&sql).execute(pool).await?;
        Self::after_create_table().await?;
        Ok(())
    }

    /// Creates indexes for the model.
    async fn create_indexes() -> Result<u64, Error> {
        let pool = Self::init_writer()?.pool();

        let table_name = Self::table_name();
        let columns = Self::columns();
        let mut rows = 0;
        if cfg!(feature = "orm-mysql") {
            let sql = format!("SHOW INDEXES FROM {table_name}");
            let indexes = sqlx::query(&sql).fetch_all(pool).await?;
            if indexes.len() > 1 {
                return Ok(0);
            }

            let mut text_search_columns = Vec::new();
            for col in columns {
                if let Some(index_type) = col.index_type() {
                    let column_name = col.name();
                    if index_type == "fulltext" || index_type == "text" {
                        text_search_columns.push(column_name);
                    } else if index_type == "unique" || index_type == "spatial" {
                        let index_type = index_type.to_uppercase();
                        let sql = format!(
                            "CREATE {index_type} INDEX {table_name}_{column_name}_index \
                                ON {table_name} ({column_name});"
                        );
                        rows = sqlx::query(&sql)
                            .execute(pool)
                            .await?
                            .rows_affected()
                            .max(rows);
                    } else if index_type == "btree" || index_type == "hash" {
                        let index_type = index_type.to_uppercase();
                        let sql = format!(
                            "CREATE INDEX {table_name}_{column_name}_index \
                                ON {table_name} ({column_name}) USING {index_type};"
                        );
                        rows = sqlx::query(&sql)
                            .execute(pool)
                            .await?
                            .rows_affected()
                            .max(rows);
                    }
                }
            }
            if !text_search_columns.is_empty() {
                let text_search_columns = text_search_columns.join(", ");
                let sql = format!(
                    "CREATE FULLTEXT INDEX {table_name}_text_search_index \
                        ON {table_name} ({text_search_columns});"
                );
                rows = sqlx::query(&sql)
                    .execute(pool)
                    .await?
                    .rows_affected()
                    .max(rows);
            }
            return Ok(rows);
        }

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
                            ON {table_name} ({column_name});"
                    );
                    rows = sqlx::query(&sql)
                        .execute(pool)
                        .await?
                        .rows_affected()
                        .max(rows);
                } else {
                    let sort_order = if index_type == "btree" { " DESC" } else { "" };
                    let sql = format!(
                        "CREATE INDEX IF NOT EXISTS {table_name}_{column_name}_index \
                            ON {table_name} USING {index_type}({column_name}{sort_order});"
                    );
                    rows = sqlx::query(&sql)
                        .execute(pool)
                        .await?
                        .rows_affected()
                        .max(rows);
                }
            }
        }
        for language in text_search_languages {
            let text = text_search_columns
                .iter()
                .filter_map(|col| (col.0 == language).then_some(col.1.as_str()))
                .intersperse(" || ' ' || ")
                .collect::<String>();
            let text_search = format!("to_tsvector('{language}', {text})");
            let sql = format!(
                "CREATE INDEX IF NOT EXISTS {table_name}_text_search_{language}_index \
                    ON {table_name} USING gin({text_search});"
            );
            rows = sqlx::query(&sql)
                .execute(pool)
                .await?
                .rows_affected()
                .max(rows);
        }
        Ok(rows)
    }

    /// Inserts the model into the table.
    async fn insert(mut self) -> Result<QueryContext, Error> {
        let pool = Self::acquire_writer().await?.pool();
        let model_data = self.before_insert().await?;

        let table_name = Self::table_name();
        let map = self.into_map();
        let fields = Self::fields().join(", ");
        let values = Self::columns()
            .iter()
            .map(|col| col.encode_value(map.get(col.name())))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("INSERT INTO {table_name} ({fields}) VALUES ({values});");

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        Self::after_insert(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            Err(Error::new(format!(
                "{rows_affected} rows are affected while it is expected to affect 1 row"
            )))
        }
    }

    /// Inserts many models into the table.
    async fn insert_many(models: Vec<Self>) -> Result<u64, Error> {
        let pool = Self::acquire_writer().await?.pool();
        let columns = Self::columns();
        let mut values = Vec::with_capacity(models.len());
        for mut model in models.into_iter() {
            let _model_data = model.before_insert().await?;

            let map = model.into_map();
            let entries = columns
                .iter()
                .map(|col| col.encode_value(map.get(col.name())))
                .collect::<Vec<_>>();
            values.push(format!("({})", entries.join(", ")));
        }

        let table_name = Self::table_name();
        let fields = Self::fields().join(", ");
        let values = values.join(", ");
        let sql = format!("INSERT INTO {table_name} ({fields}) VALUES {values};");

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        Self::after_scan(&ctx).await?;
        Ok(rows_affected)
    }

    /// Updates the model in the table.
    async fn update(mut self) -> Result<QueryContext, Error> {
        let pool = Self::acquire_writer().await?.pool();
        let model_data = self.before_update().await?;

        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let primary_key = Query::escape_string(self.primary_key());
        let map = self.into_map();
        let readonly_fields = Self::readonly_fields();
        let num_writable_fields = Self::fields().len() - readonly_fields.len();
        let mut mutations = Vec::with_capacity(num_writable_fields);
        for col in Self::columns() {
            let field = col.name();
            if !readonly_fields.contains(&field) {
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
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        Self::after_update(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            Err(Error::new(format!(
                "{rows_affected} rows are affected while it is expected to affect 1 row"
            )))
        }
    }

    /// Updates at most one model selected by the query in the table.
    async fn update_one(query: &mut Query, mutation: &mut Mutation) -> Result<QueryContext, Error> {
        let pool = Self::acquire_writer().await?.pool();
        Self::before_mutation(query, mutation).await?;

        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let filters = query.format_filters::<Self>();
        let updates = mutation.format_updates::<Self>();
        let sql = if cfg!(feature = "orm-mysql") {
            // MySQL doesn't yet support 'LIMIT & IN/ALL/ANY/SOME subquery'
            // and self-referencing in UPDATE/DELETE
            format!(
                "UPDATE {table_name} SET {updates} WHERE {primary_key_name} IN \
                    (SELECT * from (SELECT {primary_key_name} FROM {table_name} {filters}) AS t);"
            )
        } else {
            let sort = query.format_sort();
            format!(
                "UPDATE {table_name} SET {updates} WHERE {primary_key_name} IN \
                    (SELECT {primary_key_name} FROM {table_name} {filters} {sort} LIMIT 1);"
            )
        };

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected <= 1;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        Self::after_mutation(&ctx).await?;
        if success {
            Ok(ctx)
        } else {
            Err(Error::new(format!(
                "{rows_affected} rows are affected while it is expected to affect at most 1 row"
            )))
        }
    }

    /// Updates many models selected by the query in the table.
    async fn update_many(query: &mut Query, mutation: &mut Mutation) -> Result<u64, Error> {
        let pool = Self::acquire_writer().await?.pool();
        Self::before_mutation(query, mutation).await?;

        let table_name = Self::table_name();
        let filters = query.format_filters::<Self>();
        let updates = mutation.format_updates::<Self>();
        let sql = format!("UPDATE {table_name} SET {updates} {filters};");

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        Self::after_scan(&ctx).await?;
        Self::after_mutation(&ctx).await?;
        Ok(rows_affected)
    }

    /// Updates or inserts the model into the table.
    async fn upsert(mut self) -> Result<QueryContext, Error> {
        let pool = Self::acquire_writer().await?.pool();
        let model_data = self.before_upsert().await?;

        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let map = self.into_map();
        let fields = Self::fields();
        let num_fields = fields.len();
        let readonly_fields = Self::readonly_fields();
        let num_writable_fields = num_fields - readonly_fields.len();
        let mut values = Vec::with_capacity(num_fields);
        let mut mutations = Vec::with_capacity(num_writable_fields);
        for col in Self::columns() {
            let field = col.name();
            let value = col.encode_value(map.get(field));
            if !readonly_fields.contains(&field) {
                let field = Query::format_field(field);
                mutations.push(format!("{field} = {value}"));
            }
            values.push(value);
        }

        let fields = fields.join(", ");
        let values = values.join(", ");
        let mutations = mutations.join(", ");
        let sql = if cfg!(feature = "orm-mysql") {
            format!(
                "INSERT INTO {table_name} ({fields}) VALUES ({values}) \
                    ON DUPLICATE KEY UPDATE {mutations};"
            )
        } else {
            format!(
                "INSERT INTO {table_name} ({fields}) VALUES ({values}) \
                    ON CONFLICT ({primary_key_name}) DO UPDATE SET {mutations};"
            )
        };

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        Self::after_upsert(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            Err(Error::new(format!(
                "{rows_affected} rows are affected while it is expected to affect 1 row"
            )))
        }
    }

    /// Deletes the model in the table.
    async fn delete(mut self) -> Result<QueryContext, Error> {
        let pool = Self::acquire_writer().await?.pool();
        let model_data = self.before_delete().await?;

        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let primary_key = self.primary_key();
        let sql = if cfg!(feature = "orm-mysql") {
            let placeholder = Query::placeholder(1);
            format!("DELETE FROM {table_name} WHERE {primary_key_name} = {placeholder};")
        } else {
            let primary_key = Query::escape_string(primary_key);
            format!("DELETE FROM {table_name} WHERE {primary_key_name} = {primary_key};")
        };

        let mut ctx = Self::before_scan(&sql).await?;
        let query = if cfg!(feature = "orm-mysql") {
            ctx.add_argument(primary_key.to_string());
            sqlx::query(&sql).bind(primary_key.to_string())
        } else {
            sqlx::query(&sql)
        };
        let query_result = query.execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected == 1;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        self.after_delete(&ctx, model_data).await?;
        if success {
            Ok(ctx)
        } else {
            Err(Error::new(format!(
                "{rows_affected} rows are affected while it is expected to affect 1 row"
            )))
        }
    }

    /// Deletes at most one model selected by the query in the table.
    async fn delete_one(query: &mut Query) -> Result<QueryContext, Error> {
        let pool = Self::acquire_writer().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let sql = format!(
            "DELETE FROM {table_name} WHERE {primary_key_name} IN \
                (SELECT {primary_key_name} FROM {table_name} {filters} {sort} LIMIT 1);"
        );

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        let success = rows_affected <= 1;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        if success {
            Ok(ctx)
        } else {
            Err(Error::new(format!(
                "{rows_affected} rows are affected while it is expected to affect at most 1 row"
            )))
        }
    }

    /// Deletes many models selected by the query in the table.
    async fn delete_many(query: &mut Query) -> Result<u64, Error> {
        let pool = Self::acquire_writer().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let filters = query.format_filters::<Self>();
        let sql = format!("DELETE FROM {table_name} {filters};");

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = sqlx::query(&sql).execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(rows_affected)
    }

    /// Finds models selected by the query in the table,
    /// and decodes it as `Vec<T>`.
    async fn find<T: DecodeRow<DatabaseRow, Error = Error>>(
        query: &mut Query,
    ) -> Result<Vec<T>, Error> {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} {pagination};");

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while let Some(row) = rows.try_next().await? && max_rows > 0 {
            data.push(T::decode_row(&row)?);
            max_rows -= 1;
        }
        ctx.set_query(&sql);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Finds models selected by the query in the table,
    /// and parses it as `Vec<T>`.
    async fn find_as<T: DeserializeOwned>(query: &mut Query) -> Result<Vec<T>, Error> {
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
    async fn find_one<T: DecodeRow<DatabaseRow, Error = Error>>(
        query: &mut Query,
    ) -> Result<Option<T>, Error> {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} LIMIT 1;");

        let mut ctx = Self::before_scan(&sql).await?;
        let (num_rows, data) = if let Some(row) = sqlx::query(&sql).fetch_optional(pool).await? {
            (1, Some(T::decode_row(&row)?))
        } else {
            (0, None)
        };
        ctx.set_query(sql);
        ctx.set_query_result(Some(num_rows), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Finds one model selected by the query in the table,
    /// and parses it as an instance of type `T`.
    async fn find_one_as<T: DeserializeOwned>(query: &mut Query) -> Result<Option<T>, Error> {
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

    /// Finds a value selected by the query in the table,
    /// and decodes it as a single concrete type `T`.
    async fn find_scalar<T>(query: &mut Query) -> Result<T, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} LIMIT 1;");

        let mut ctx = Self::before_scan(&sql).await?;
        let scalar = sqlx::query_scalar(&sql).fetch_one(pool).await?;
        ctx.set_query(sql);
        ctx.set_query_result(Some(1), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(scalar)
    }

    /// Finds a list of scalar values selected by the query in the table,
    /// and decodes it as a `Vec<T>`.
    async fn find_scalars<T>(query: &mut Query) -> Result<Vec<T>, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} {pagination};");

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while let Some(row) = rows.try_next().await? && max_rows > 0 {
            data.push(row.try_get_unchecked(0)?);
            max_rows -= 1;
        }
        ctx.set_query(&sql);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Populates the related data in the corresponding `columns` for `Vec<Map>` using
    /// a merged select on the primary key, which solves the `N+1` problem.
    async fn populate<const N: usize>(
        query: &mut Query,
        data: &mut Vec<Map>,
        columns: [&str; N],
    ) -> Result<u64, Error> {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut values = Vec::new();
        for row in data.iter() {
            for col in columns {
                if let Some(mut vec) = row.parse_str_array(col) {
                    values.append(&mut vec);
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

        let table_name = Self::table_name();
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT {projection} FROM {table_name} {filters};");

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut associations = Map::with_capacity(num_values);
        let translate_enabled = query.translate_enabled();
        while let Some(row) = rows.try_next().await? {
            let mut map = Map::decode_row(&row)?;
            let primary_key_value = map
                .get_str(primary_key_name)
                .map(|s| s.to_owned())
                .unwrap_or_default();
            Self::after_decode(&mut map).await?;
            translate_enabled.then(|| Self::translate_model(&mut map));
            associations.upsert(primary_key_value, map);
        }
        ctx.set_query(&sql);
        ctx.set_query_result(Some(u64::try_from(associations.len())?), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;

        for row in data {
            for col in columns {
                if let Some(value) = row.get_mut(col) {
                    if let Some(value) = value.as_str() {
                        if let Some(value) = associations.get(value) {
                            row.upsert(col, value.clone());
                        }
                    } else if let Some(entries) = value.as_array_mut() {
                        for entry in entries {
                            if let Some(value) = entry.as_str() {
                                if let Some(value) = associations.get(value) {
                                    *entry = value.clone();
                                }
                            }
                        }
                    }
                }
            }
        }
        u64::try_from(associations.len()).map_err(Error::from)
    }

    /// Populates the related data in the corresponding `columns` for `Map` using
    /// a merged select on the primary key, which solves the `N+1` problem.
    async fn populate_one<const N: usize>(
        query: &mut Query,
        data: &mut Map,
        columns: [&str; N],
    ) -> Result<(), Error> {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut values = Vec::new();
        for col in columns {
            if let Some(mut vec) = data.parse_str_array(col) {
                values.append(&mut vec);
            }
        }

        let num_values = values.len();
        if num_values > 0 {
            let primary_key_values = Map::from_entry("$in", values);
            query.add_filter(primary_key_name, primary_key_values);
        } else {
            return Ok(());
        }

        let table_name = Self::table_name();
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT {projection} FROM {table_name} {filters};");

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut associations = Map::with_capacity(num_values);
        let translate_enabled = query.translate_enabled();
        while let Some(row) = rows.try_next().await? {
            let mut map = Map::decode_row(&row)?;
            let primary_key_value = map
                .get_str(primary_key_name)
                .map(|s| s.to_owned())
                .unwrap_or_default();
            Self::after_decode(&mut map).await?;
            translate_enabled.then(|| Self::translate_model(&mut map));
            associations.upsert(primary_key_value, map);
        }
        ctx.set_query(&sql);
        ctx.set_query_result(Some(u64::try_from(associations.len())?), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;

        for col in columns {
            if let Some(value) = data.get_mut(col) {
                if let Some(value) = value.as_str() {
                    if let Some(value) = associations.get(value) {
                        data.upsert(col, value.clone());
                    }
                } else if let Some(entries) = value.as_array_mut() {
                    for entry in entries {
                        if let Some(value) = entry.as_str() {
                            if let Some(value) = associations.get(value) {
                                *entry = value.clone();
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Performs a left outer join to another table to filter rows in the "joined" table,
    /// and decodes it as `Vec<T>`.
    async fn lookup<M: Schema, T: DecodeRow<DatabaseRow, Error = Error>>(
        query: &mut Query,
        left_columns: &[&str],
        right_columns: &[&str],
    ) -> Result<Vec<T>, Error> {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let model_name = Query::format_field(Self::model_name());
        let other_table_name = M::table_name();
        let other_model_name = Query::format_field(M::model_name());
        let projection = query.format_fields();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let on_expressions = left_columns
            .iter()
            .zip(right_columns.iter())
            .map(|(left_col, right_col)| {
                let left_col = Query::format_field(left_col);
                let right_col = Query::format_field(right_col);
                format!("{model_name}.{left_col} = {other_model_name}.{right_col}")
            })
            .collect::<Vec<_>>()
            .join(" AND ");
        let sql = format!(
            "SELECT {projection} FROM {table_name} {model_name} \
                LEFT OUTER JOIN {other_table_name} {other_model_name} \
                    ON {on_expressions} {filters} {sort} {pagination};"
        );

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while let Some(row) = rows.try_next().await? && max_rows > 0 {
            data.push(T::decode_row(&row)?);
            max_rows -= 1;
        }
        ctx.set_query(&sql);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Performs a left outer join to another table to filter rows in the "joined" table,
    /// and parses it as `Vec<T>`.
    async fn lookup_as<M: Schema, T: DeserializeOwned>(
        query: &mut Query,
        left_columns: &[&str],
        right_columns: &[&str],
    ) -> Result<Vec<T>, Error> {
        let mut data = Self::lookup::<M, Map>(query, left_columns, right_columns).await?;
        let translate_enabled = query.translate_enabled();
        for model in data.iter_mut() {
            Self::after_decode(model).await?;
            translate_enabled.then(|| Self::translate_model(model));
        }
        serde_json::from_value(data.into()).map_err(Error::from)
    }

    /// Counts the number of rows selected by the query in the table.
    async fn count(query: &mut Query) -> Result<u64, Error> {
        let pool = Self::acquire_writer().await?.pool();
        Self::before_count(query).await?;

        let table_name = Self::table_name();
        let filters = query.format_filters::<Self>();
        let sql = format!("SELECT count(*) FROM {table_name} {filters};");

        let mut ctx = Self::before_scan(&sql).await?;
        let count: i64 = sqlx::query_scalar(&sql).fetch_one(pool).await?;
        ctx.set_query(sql);
        ctx.set_query_result(Some(1), true);
        Self::after_scan(&ctx).await?;
        Self::after_count(&ctx).await?;
        u64::try_from(count).map_err(Error::from)
    }

    /// Counts the number of rows selected by the query in the table.
    /// The boolean value determines whether it only counts distinct values or not.
    async fn count_many<T: DecodeRow<DatabaseRow, Error = Error>>(
        query: &mut Query,
        columns: &[(&str, bool)],
    ) -> Result<T, Error> {
        let pool = Self::acquire_writer().await?.pool();
        Self::before_count(query).await?;

        let table_name = Self::table_name();
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
        let row = sqlx::query(&sql).fetch_one(pool).await?;
        ctx.set_query(sql);
        ctx.set_query_result(Some(1), true);
        Self::after_scan(&ctx).await?;
        Self::after_count(&ctx).await?;
        T::decode_row(&row).map_err(Error::from)
    }

    /// Counts the number of rows selected by the query in the table,
    /// and parses it as an instance of type `T`.
    async fn count_many_as<T: DeserializeOwned>(
        query: &mut Query,
        columns: &[(&str, bool)],
    ) -> Result<T, Error> {
        let map = Self::count_many::<Map>(query, columns).await?;
        serde_json::from_value(map.into()).map_err(Error::from)
    }

    /// Executes the query in the table, and returns the total number of rows affected.
    async fn execute(query: &str, params: Option<&Map>) -> Result<u64, Error> {
        let pool = Self::acquire_reader().await?.pool();
        let (sql, values) = Query::prepare_query(query, params);
        let mut query = sqlx::query(&sql);
        let mut arguments = Vec::with_capacity(values.len());
        for value in values {
            query = query.bind(value.to_string());
            arguments.push(value.to_string());
        }

        let mut ctx = Self::before_scan(&sql).await?;
        let query_result = query.execute(pool).await?;
        let rows_affected = query_result.rows_affected();
        ctx.set_query(sql);
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(Some(rows_affected), true);
        Self::after_scan(&ctx).await?;
        Ok(rows_affected)
    }

    /// Executes the query in the table, and decodes it as `Vec<T>`.
    async fn query<T: DecodeRow<DatabaseRow, Error = Error>>(
        query: &str,
        params: Option<&Map>,
    ) -> Result<Vec<T>, Error> {
        let pool = Self::acquire_reader().await?.pool();
        let (sql, values) = Query::prepare_query(query, params);
        let mut query = sqlx::query(&sql);
        let mut arguments = Vec::with_capacity(values.len());
        for value in values {
            query = query.bind(value.to_string());
            arguments.push(value.to_string());
        }

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = query.fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while let Some(row) = rows.try_next().await? && max_rows > 0 {
            data.push(T::decode_row(&row)?);
            max_rows -= 1;
        }
        ctx.set_query(sql.as_ref());
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
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
    async fn query_one<T: DecodeRow<DatabaseRow, Error = Error>>(
        query: &str,
        params: Option<&Map>,
    ) -> Result<Option<T>, Error> {
        let pool = Self::acquire_reader().await?.pool();
        let (sql, values) = Query::prepare_query(query, params);
        let mut query = sqlx::query(&sql);
        let mut arguments = Vec::with_capacity(values.len());
        for value in values {
            query = query.bind(value.to_string());
            arguments.push(value.to_string());
        }

        let mut ctx = Self::before_scan(&sql).await?;
        let (num_rows, data) = if let Some(row) = query.fetch_optional(pool).await? {
            (1, Some(T::decode_row(&row)?))
        } else {
            (0, None)
        };
        ctx.set_query(sql);
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(Some(num_rows), true);
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

    /// Executes the query in the table, and decodes it as a single concrete type `T`.
    async fn query_scalar<T>(query: &str, params: Option<&Map>) -> Result<T, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();
        let (sql, values) = Query::prepare_query(query, params);
        let mut query = sqlx::query_scalar(&sql);
        let mut arguments = Vec::with_capacity(values.len());
        for value in values {
            query = query.bind(value.to_string());
            arguments.push(value.to_string());
        }

        let mut ctx = Self::before_scan(&sql).await?;
        let scalar = query.fetch_one(pool).await?;
        ctx.set_query(sql);
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(Some(1), true);
        Self::after_scan(&ctx).await?;
        Ok(scalar)
    }

    /// Executes the query in the table, and decodes the scalar values as `Vec<T>`.
    async fn query_scalars<T>(query: &str, params: Option<&Map>) -> Result<Vec<T>, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();
        let (sql, values) = Query::prepare_query(query, params);
        let mut query = sqlx::query(&sql);
        let mut arguments = Vec::with_capacity(values.len());
        for value in values {
            query = query.bind(value.to_string());
            arguments.push(value.to_string());
        }

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = query.fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while let Some(row) = rows.try_next().await? && max_rows > 0 {
            data.push(row.try_get_unchecked(0)?);
            max_rows -= 1;
        }
        ctx.set_query(sql.as_ref());
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
        Self::after_scan(&ctx).await?;
        Ok(data)
    }

    /// Executes the specific operations inside a transaction.
    /// If the operations return an error, the transaction will be rolled back;
    /// if not, the transaction will be committed.
    async fn transaction<F, T>(tx: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(&'a Transaction<DatabaseDriver>) -> BoxFuture<'a, Result<T, Error>>,
    {
        let pool = Self::acquire_writer().await?.pool();
        let transaction = pool.begin().await?;
        let data = tx(&transaction).await?;
        transaction.commit().await?;
        Ok(data)
    }

    /// Finds one model selected by the primary key in the table,
    /// and decodes it as an instance of type `T`.
    async fn find_by_id<T: DecodeRow<DatabaseRow, Error = Error>>(
        primary_key: &Self::PrimaryKey,
    ) -> Result<Option<T>, Error> {
        let pool = Self::acquire_reader().await?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let query = Self::default_query();
        let projection = query.format_fields();
        let sql = if cfg!(feature = "orm-mysql") {
            let placeholder = Query::placeholder(1);
            format!(
                "SELECT {projection} FROM {table_name} WHERE {primary_key_name} = {placeholder};"
            )
        } else {
            let primary_key = Query::escape_string(primary_key);
            format!(
                "SELECT {projection} FROM {table_name} WHERE {primary_key_name} = {primary_key};"
            )
        };

        let mut ctx = Self::before_scan(&sql).await?;
        let query = if cfg!(feature = "orm-mysql") {
            ctx.add_argument(primary_key.to_string());
            sqlx::query(&sql).bind(primary_key.to_string())
        } else {
            sqlx::query(&sql)
        };
        let (num_rows, data) = if let Some(row) = query.fetch_optional(pool).await? {
            (1, Some(T::decode_row(&row)?))
        } else {
            (0, None)
        };
        ctx.set_query(sql);
        ctx.set_query_result(Some(num_rows), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
    }

    /// Finds one model selected by the primary key in the table, and parses it as `Self`.
    async fn try_get_model(primary_key: &Self::PrimaryKey) -> Result<Self, Error> {
        let pool = Self::acquire_reader().await?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let query = Self::default_query();
        let projection = query.format_fields();
        let sql = if cfg!(feature = "orm-mysql") {
            let placeholder = Query::placeholder(1);
            format!(
                "SELECT {projection} FROM {table_name} WHERE {primary_key_name} = {placeholder};"
            )
        } else {
            let primary_key = Query::escape_string(primary_key);
            format!(
                "SELECT {projection} FROM {table_name} WHERE {primary_key_name} = {primary_key};"
            )
        };

        let mut ctx = Self::before_scan(&sql).await?;
        let query = if cfg!(feature = "orm-mysql") {
            ctx.add_argument(primary_key.to_string());
            sqlx::query(&sql).bind(primary_key.to_string())
        } else {
            sqlx::query(&sql)
        };
        if let Some(row) = query.fetch_optional(pool).await? {
            ctx.set_query(sql);
            ctx.set_query_result(Some(1), true);
            Self::after_scan(&ctx).await?;
            Self::after_query(&ctx).await?;

            let mut map = Map::decode_row(&row)?;
            Self::after_decode(&mut map).await?;
            Self::try_from_map(map).map_err(Error::from)
        } else {
            ctx.set_query(sql);
            ctx.set_query_result(Some(0), true);
            Self::after_scan(&ctx).await?;
            Self::after_query(&ctx).await?;

            let model_name = Self::MODEL_NAME;
            Err(Error::new(format!(
                "404 Not Found: no rows for the model `{model_name}` with the key `{primary_key}`"
            )))
        }
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

        let data = Self::find::<Map>(&mut query).await?;
        let mut primary_key_values = Vec::with_capacity(data.len());
        for map in data.into_iter() {
            for (_key, value) in map.into_iter() {
                primary_key_values.push(value);
            }
        }
        Ok(primary_key_values)
    }

    /// Returns `true` if the model is unique on the column values.
    async fn is_unique_on<const N: usize>(
        &self,
        columns: [(&str, JsonValue); N],
    ) -> Result<bool, Error> {
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut query = Query::default();
        let mut fields = vec![primary_key_name];
        for (field, value) in columns.into_iter() {
            fields.push(field);
            query.add_filter(field, value);
        }
        query.allow_fields(&fields);
        query.set_limit(2);

        let data = Self::find::<Map>(&mut query).await?;
        match data.len() {
            0 => Ok(true),
            1 => {
                if let Some(value) = data.first().and_then(|m| m.get_str(primary_key_name)) {
                    Ok(self.primary_key().to_string() == value)
                } else {
                    Ok(true)
                }
            }
            _ => Ok(false),
        }
    }
}
