use crate::{
    database::{Column, ConnectionPool, Model, Mutation, Query},
    request::Validation,
    Map,
};
use futures::TryStreamExt;
use serde::de::DeserializeOwned;
use serde_json::json;
use sqlx::{Error, Row};

/// Model schema.
pub trait Schema: 'static + Send + Sync + Model {
    /// Type name as a str.
    const TYPE_NAME: &'static str;
    /// Primary key name as a str.
    const PRIMARY_KEY_NAME: &'static str = "id";
    /// Reader name.
    const READER_NAME: &'static str = "main";
    /// Writer name.
    const WRITER_NAME: &'static str = "main";

    /// Returns a reference to the columns.
    fn columns() -> &'static [Column<'static>];

    /// Returns the primary key value as a `String`.
    fn primary_key(&self) -> String;

    /// Gets the model reader.
    async fn get_reader() -> Option<&'static ConnectionPool>;

    /// Gets the model writer.
    async fn get_writer() -> Option<&'static ConnectionPool>;

    /// Returns the model name.
    #[inline]
    fn model_name() -> &'static str {
        Self::TYPE_NAME
    }

    /// Returns the model namespace.
    #[inline]
    fn model_namespace() -> &'static str {
        [*super::NAMESPACE_PREFIX, Self::TYPE_NAME].join(":").leak()
    }

    /// Returns the table name.
    #[inline]
    fn table_name() -> &'static str {
        [*super::NAMESPACE_PREFIX, Self::TYPE_NAME]
            .join("_")
            .replace(':', "_")
            .leak()
    }

    /// Gets a column for the field.
    #[inline]
    fn get_column(key: &str) -> Option<&Column<'static>> {
        Self::columns().iter().find(|col| col.name() == key)
    }

    /// Initializes the model reader.
    #[inline]
    fn init_reader() -> Result<&'static ConnectionPool, Error> {
        super::SHARED_CONNECTION_POOLS
            .get_pool(Self::READER_NAME)
            .ok_or(Error::PoolClosed)
    }

    /// Initializes the model writer.
    #[inline]
    fn init_writer() -> Result<&'static ConnectionPool, Error> {
        super::SHARED_CONNECTION_POOLS
            .get_pool(Self::WRITER_NAME)
            .ok_or(Error::PoolClosed)
    }

    /// Creates table for the model.
    async fn create_table() -> Result<u64, Error> {
        let pool = Self::init_writer()?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut columns = Vec::new();
        for col in Self::columns() {
            let name = col.name();
            let postgres_type = col.postgres_type();
            let mut column = format!("{name} {postgres_type}");
            if let Some(value) = col.default_value() {
                column = column + " DEFAULT " + &col.format_postgres_value(value);
            } else if col.is_not_null() {
                column += " NOT NULL";
            }
            columns.push(column);
        }

        let columns = columns.join(",\n");
        let sql = format!(
            "
                CREATE TABLE IF NOT EXISTS {table_name} (
                    {columns},
                    CONSTRAINT {table_name}_pkey PRIMARY KEY ({primary_key_name})
                );
            "
        );
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Creates indexes for the model.
    async fn create_indexes() -> Result<u64, Error> {
        let pool = Self::init_writer()?.pool();
        let table_name = Self::table_name();
        let mut text_search_languages = Vec::new();
        let mut text_search_columns = Vec::new();
        let mut rows = 0;
        for col in Self::columns() {
            if let Some(index_type) = col.index_type() {
                let column_name = col.name();
                if index_type.starts_with("text") {
                    let language = index_type.strip_prefix("text:").unwrap_or("english");
                    let column = format!("coalesce({column_name}, '')");
                    text_search_languages.push(language);
                    text_search_columns.push((language, column));
                } else {
                    let sort_order = if index_type == "btree" { " DESC" } else { "" };
                    let sql = format!(
                        "
                            CREATE INDEX CONCURRENTLY IF NOT EXISTS {table_name}_{column_name}_index
                            ON {table_name} USING {index_type}({column_name}{sort_order});
                        "
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
            let column = text_search_columns
                .iter()
                .filter_map(|col| (col.0 == language).then_some(col.1.as_str()))
                .intersperse(" || ' ' || ")
                .collect::<String>();
            let text_search = format!("to_tsvector('{language}', {column})");
            let sql = format!(
                "
                    CREATE INDEX CONCURRENTLY IF NOT EXISTS {table_name}_text_search_{language}_index
                    ON {table_name} USING gin({text_search});
                "
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
    async fn insert(self) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let map = self.into_map();
        let mut columns = Vec::new();
        let mut values = Vec::new();
        for col in Self::columns() {
            let column = col.name();
            let value = col.encode_postgres_value(map.get(column));
            columns.push(column);
            values.push(value);
        }

        let columns = columns.join(",");
        let values = values.join(",");
        let sql = format!("INSERT INTO {table_name} ({columns}) VALUES ({values});");
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Inserts many models into the table.
    async fn insert_many(models: Vec<Self>) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let mut columns = Vec::new();
        let mut values = Vec::new();
        for model in models.into_iter() {
            let map = model.into_map();
            let mut entries = Vec::new();
            for col in Self::columns() {
                let column = col.name();
                let value = col.encode_postgres_value(map.get(column));
                columns.push(column);
                entries.push(value);
            }
            values.push(format!("({})", entries.join(",")));
        }

        let columns = columns.join(",");
        let values = values.join(",");
        let sql = format!("INSERT INTO {table_name} ({columns}) VALUES ({values});");
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Updates the model in the table.
    async fn update(self) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let primary_key = self.primary_key();
        let map = self.into_map();
        let mut mutations = Vec::new();
        for col in Self::columns() {
            let column = col.name();
            if column != primary_key_name {
                let value = col.encode_postgres_value(map.get(column));
                mutations.push(format!("{column} = {value}"));
            }
        }

        let mutations = mutations.join(",");
        let sql = format!(
            "UPDATE {table_name} SET {mutations} WHERE {primary_key_name} = '{primary_key}';"
        );
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Updates at most one model selected by the query in the table.
    async fn update_one(query: Query, mutation: Mutation) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let filter = query.format_filter::<Self>();
        let sort = query.format_sort();
        let update = mutation.format_update::<Self>();
        let sql = format!(
            "
                UPDATE {table_name} {update} WHERE {primary_key_name} IN
                (SELECT {primary_key_name} FROM {table_name} {filter} {sort} LIMIT 1);
            "
        );
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Updates many models selected by the query in the table.
    async fn update_many(query: Query, mutation: Mutation) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let filter = query.format_filter::<Self>();
        let update = mutation.format_update::<Self>();
        let sql = format!("UPDATE {table_name} {update} {filter};");
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Updates or inserts the model into the table.
    async fn upsert(self) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let map = self.into_map();
        let mut columns = Vec::new();
        let mut values = Vec::new();
        let mut mutations = Vec::new();
        for col in Self::columns() {
            let column = col.name();
            let value = col.encode_postgres_value(map.get(column));
            if column != primary_key_name {
                mutations.push(format!("{column} = {value}"));
            }
            columns.push(column);
            values.push(value);
        }

        let columns = columns.join(",");
        let values = values.join(",");
        let mutations = mutations.join(",");
        let sql = format!(
            "
                INSERT INTO {table_name} ({columns}) VALUES ({values})
                ON CONFLICT ({primary_key_name}) DO UPDATE SET {mutations};
            "
        );
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Deletes the model in the table.
    async fn delete(self) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let primary_key = self.primary_key();
        let sql = format!("DELETE FROM {table_name} WHERE {primary_key_name} = '{primary_key}';");
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Deletes at most one model selected by the query in the table.
    async fn delete_one(query: Query) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let filter = query.format_filter::<Self>();
        let sort = query.format_sort();
        let sql = format!(
            "
                DELETE FROM {table_name} WHERE {primary_key_name} IN
                (SELECT {primary_key_name} FROM {table_name} {filter} {sort} LIMIT 1);
            "
        );
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Deletes many models selected by the query in the table.
    async fn delete_many(query: Query) -> Result<u64, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let filter = query.format_filter::<Self>();
        let sql = format!("DELETE FROM {table_name} {filter};");
        let query_result = sqlx::query(&sql).execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Finds models selected by the query in the table, and parses it as `Vec<Map>`.
    async fn find(query: Query) -> Result<Vec<Map>, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let fields = query.fields();
        let projection = query.format_fields();
        let filter = query.format_filter::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let sql = format!("SELECT {projection} FROM {table_name} {filter} {sort} {pagination};");
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut data = Vec::new();
        if fields.is_empty() {
            let columns = Self::columns();
            let capacity = columns.len();
            while let Some(row) = rows.try_next().await? {
                let mut map = Map::with_capacity(capacity);
                for col in columns {
                    let value = col.decode_postgres_row(&row)?;
                    map.insert(col.name().to_owned(), value);
                }
                data.push(map);
            }
        } else {
            while let Some(row) = rows.try_next().await? {
                let map = Column::parse_postgres_row(&row)?;
                data.push(map);
            }
        }
        Ok(data)
    }

    /// Finds models selected by the query in the table, and parses it as `Vec<T>`.
    async fn find_as<T: DeserializeOwned>(query: Query) -> Result<Vec<T>, Error> {
        let data = Self::find(query).await?;
        serde_json::from_value(data.into()).map_err(|err| Error::Decode(Box::new(err)))
    }

    /// Finds one model selected by the query in the table, and parses it as a `Map`.
    async fn find_one(query: Query) -> Result<Option<Map>, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let fields = query.fields();
        let projection = query.format_fields();
        let filter = query.format_filter::<Self>();
        let sort = query.format_sort();
        let sql = format!("SELECT {projection} FROM {table_name} {filter} {sort} LIMIT 1;");
        let data = match sqlx::query(&sql).fetch_optional(pool).await? {
            Some(row) => {
                if fields.is_empty() {
                    let columns = Self::columns();
                    let mut map = Map::with_capacity(columns.len());
                    for col in columns {
                        let value = col.decode_postgres_row(&row)?;
                        map.insert(col.name().to_owned(), value);
                    }
                    Some(map)
                } else {
                    let map = Column::parse_postgres_row(&row)?;
                    Some(map)
                }
            }
            None => None,
        };
        Ok(data)
    }

    /// Finds one model selected by the query in the table, and parses it as an instance of type `T`.
    async fn find_one_as<T: DeserializeOwned>(query: Query) -> Result<Option<T>, Error> {
        match Self::find_one(query).await? {
            Some(data) => {
                serde_json::from_value(data.into()).map_err(|err| Error::Decode(Box::new(err)))
            }
            None => Ok(None),
        }
    }

    /// Fetches the associated data in the corresponding `columns` for `Vec<Map>` using
    /// a merged select on the primary key, which solves the `N+1` problem.
    async fn fetch<const N: usize>(
        mut query: Query,
        data: &mut Vec<Map>,
        columns: [&str; N],
    ) -> Result<u64, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut values: Vec<String> = Vec::new();
        for row in data.iter() {
            for col in columns {
                if let Some(mut vec) = Validation::parse_array(row.get(col)) {
                    values.append(&mut vec);
                }
            }
        }
        if !values.is_empty() {
            let mut primary_key_filter = Map::new();
            primary_key_filter.insert(
                primary_key_name.to_owned(),
                json!({
                    "$in": values,
                }),
            );
            query.append_filter(&mut primary_key_filter);
        }

        let fields = query.fields();
        let projection = query.format_fields();
        let filter = query.format_filter::<Self>();
        let sql = format!("SELECT {projection} FROM {table_name} {filter};");
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut associations = Map::new();
        if fields.is_empty() {
            let columns = Self::columns();
            let capacity = columns.len();
            while let Some(row) = rows.try_next().await? {
                let primary_key_value = row.try_get_unchecked::<String, _>(primary_key_name)?;
                let mut map = Map::with_capacity(capacity);
                for col in columns {
                    let value = col.decode_postgres_row(&row)?;
                    map.insert(col.name().to_owned(), value);
                }
                associations.insert(primary_key_value, map.into());
            }
        } else {
            while let Some(row) = rows.try_next().await? {
                let primary_key_value = row.try_get_unchecked::<String, _>(primary_key_name)?;
                let map = Column::parse_postgres_row(&row)?;
                associations.insert(primary_key_value, map.into());
            }
        }
        for row in data {
            for col in columns {
                if let Some(value) = row.get_mut(col) {
                    if let Some(value) = value.as_str() {
                        if let Some(value) = associations.get(value) {
                            row.insert(col.to_owned(), value.clone());
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
        u64::try_from(associations.len()).map_err(|err| Error::Decode(Box::new(err)))
    }

    /// Fetches the associated data in the corresponding `columns` for `Map` using
    /// a merged select on the primary key, which solves the `N+1` problem.
    async fn fetch_one<const N: usize>(
        mut query: Query,
        data: &mut Map,
        columns: [&str; N],
    ) -> Result<u64, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let mut values: Vec<String> = Vec::new();
        for col in columns {
            if let Some(mut vec) = Validation::parse_array(data.get(col)) {
                values.append(&mut vec);
            }
        }
        if !values.is_empty() {
            let mut primary_key_filter = Map::new();
            primary_key_filter.insert(
                primary_key_name.to_owned(),
                json!({
                    "$in": values,
                }),
            );
            query.append_filter(&mut primary_key_filter);
        }

        let fields = query.fields();
        let projection = query.format_fields();
        let filter = query.format_filter::<Self>();
        let sql = format!("SELECT {projection} FROM {table_name} {filter};");
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut associations = Map::new();
        if fields.is_empty() {
            let columns = Self::columns();
            let capacity = columns.len();
            while let Some(row) = rows.try_next().await? {
                let primary_key_value = row.try_get_unchecked::<String, _>(primary_key_name)?;
                let mut map = Map::with_capacity(capacity);
                for col in columns {
                    let value = col.decode_postgres_row(&row)?;
                    map.insert(col.name().to_owned(), value);
                }
                associations.insert(primary_key_value, map.into());
            }
        } else {
            while let Some(row) = rows.try_next().await? {
                let primary_key_value = row.try_get_unchecked::<String, _>(primary_key_name)?;
                let map = Column::parse_postgres_row(&row)?;
                associations.insert(primary_key_value, map.into());
            }
        }
        for col in columns {
            if let Some(value) = data.get_mut(col) {
                if let Some(value) = value.as_str() {
                    if let Some(value) = associations.get(value) {
                        data.insert(col.to_owned(), value.clone());
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
        u64::try_from(associations.len()).map_err(|err| Error::Decode(Box::new(err)))
    }

    /// Counts the number of rows selected by the query in the table.
    /// The boolean value `true` denotes that it only counts distinct values in the column.
    async fn count<const N: usize>(query: Query, columns: [(&str, bool); N]) -> Result<Map, Error> {
        let pool = Self::get_writer().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let filter = query.format_filter::<Self>();
        let projection = columns
            .into_iter()
            .map(|(key, distinct)| {
                if key != "*" {
                    if distinct {
                        format!("count(distinct {key}) as {key}_count_distinct")
                    } else {
                        format!("count({key}) as {key}_count")
                    }
                } else {
                    "count(*)".to_owned()
                }
            })
            .intersperse(",".to_owned())
            .collect::<String>();
        let sql = format!("SELECT {projection} FROM {table_name} {filter};");
        let row = sqlx::query(&sql).fetch_one(pool).await?;
        let map = Column::parse_postgres_row(&row)?;
        Ok(map)
    }

    /// Executes the query in the table, and returns the total number of rows affected.
    async fn execute<const N: usize>(sql: &str, params: Option<[&str; N]>) -> Result<u64, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let mut query = sqlx::query(sql);
        if let Some(params) = params {
            for param in params {
                query = query.bind(param);
            }
        }
        let query_result = query.execute(pool).await?;
        Ok(query_result.rows_affected())
    }

    /// Executes the query in the table, and parses it as `Vec<Map>`.
    async fn query<const N: usize>(
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Vec<Map>, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let mut query = sqlx::query(sql);
        if let Some(params) = params {
            for param in params {
                query = query.bind(param);
            }
        }
        let mut rows = query.fetch(pool);
        let mut data = Vec::new();
        while let Some(row) = rows.try_next().await? {
            let map = Column::parse_postgres_row(&row)?;
            data.push(map);
        }
        Ok(data)
    }

    /// Executes the query in the table, and parses it as `Vec<T>`.
    async fn query_as<T: DeserializeOwned, const N: usize>(
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Vec<T>, Error> {
        let data = Self::query(sql, params).await?;
        serde_json::from_value(data.into()).map_err(|err| Error::Decode(Box::new(err)))
    }

    /// Executes the query in the table, and parses it as a `Map`.
    async fn query_one<const N: usize>(
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Option<Map>, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let mut query = sqlx::query(sql);
        if let Some(params) = params {
            for param in params {
                query = query.bind(param);
            }
        }
        let data = match query.fetch_optional(pool).await? {
            Some(row) => {
                let map = Column::parse_postgres_row(&row)?;
                Some(map)
            }
            None => None,
        };
        Ok(data)
    }

    /// Executes the query in the table, and parses it as an instance of type `T`.
    async fn query_one_as<T: DeserializeOwned, const N: usize>(
        sql: &str,
        params: Option<[&str; N]>,
    ) -> Result<Option<T>, Error> {
        match Self::query_one(sql, params).await? {
            Some(data) => {
                serde_json::from_value(data.into()).map_err(|err| Error::Decode(Box::new(err)))
            }
            None => Ok(None),
        }
    }

    /// Finds one model selected by the primary key in the table, and parses it as `Self`.
    async fn try_get_model(primary_key: &str) -> Result<Self, Error> {
        let pool = Self::get_reader().await.ok_or(Error::PoolClosed)?.pool();
        let table_name = Self::table_name();
        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let primary_key = Column::format_postgres_string(primary_key);
        let sql = format!(
            "
                SELECT * FROM {table_name} WHERE {primary_key_name} = {primary_key};
            "
        );
        match sqlx::query(&sql).fetch_optional(pool).await? {
            Some(row) => {
                let map = Column::parse_postgres_row(&row)?;
                serde_json::from_value(map.into()).map_err(|err| Error::Decode(Box::new(err)))
            }
            None => Err(Error::RowNotFound),
        }
    }
}
