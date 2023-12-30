use super::{column::ColumnExt, query::QueryExt, schema::Schema, DatabaseDriver};
use crate::{error::Error, extension::JsonValueExt, model::Query, Map};
use futures::TryStreamExt;
use sqlx::{Decode, Row, Type};
use std::{fmt::Display, sync::atomic::Ordering::Relaxed};

/// Query on scalar values.
pub trait ScalarQuery<K>: Schema<PrimaryKey = K>
where
    K: Default + Display + PartialEq,
{
    /// Finds a value selected by the query in the table,
    /// and decodes it as a single concrete type `T`.
    async fn find_scalar<T>(query: &Query) -> Result<T, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = Self::table_name();
        let projection = query.format_projection();
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
    async fn find_scalars<T>(query: &Query) -> Result<Vec<T>, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();
        Self::before_query(query).await?;

        let table_name = query.format_table_name::<Self>();
        let projection = query.format_table_fields::<Self>();
        let filters = query.format_filters::<Self>();
        let sort = query.format_sort();
        let pagination = query.format_pagination();
        let sql = format!("SELECT {projection} FROM {table_name} {filters} {sort} {pagination};");

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = sqlx::query(&sql).fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while max_rows > 0 {
            if let Some(row) = rows.try_next().await? {
                data.push(row.try_get_unchecked(0)?);
                max_rows -= 1;
            } else {
                break;
            }
        }
        ctx.set_query(&sql);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(data)
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
            query = query.bind(value.to_string_unquoted());
            arguments.push(value.to_string_unquoted());
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
            query = query.bind(value.to_string_unquoted());
            arguments.push(value.to_string_unquoted());
        }

        let mut ctx = Self::before_scan(&sql).await?;
        let mut rows = query.fetch(pool);
        let mut data = Vec::new();
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        while max_rows > 0 {
            if let Some(row) = rows.try_next().await? {
                data.push(row.try_get_unchecked(0)?);
                max_rows -= 1;
            } else {
                break;
            }
        }
        ctx.set_query(sql.as_ref());
        ctx.append_arguments(&mut arguments);
        ctx.set_query_result(Some(u64::try_from(data.len())?), true);
        Self::after_scan(&ctx).await?;
        Ok(data)
    }

    /// Finds a model selected by the primary key in the table,
    /// and decodes the column value as a single concrete type `T`.
    async fn find_scalar_by_id<T>(primary_key: &Self::PrimaryKey, column: &str) -> Result<T, Error>
    where
        T: Send + Unpin + Type<DatabaseDriver> + for<'r> Decode<'r, DatabaseDriver>,
    {
        let pool = Self::acquire_reader().await?.pool();

        let primary_key_name = Self::PRIMARY_KEY_NAME;
        let table_name = Self::table_name();
        let projection = Query::format_field(column);
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
        let query = sqlx::query_scalar(&sql).bind(primary_key.to_string());
        let scalar = query.fetch_one(pool).await?;
        ctx.set_query(sql);
        ctx.set_query_result(Some(1), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;
        Ok(scalar)
    }
}

impl<M, K> ScalarQuery<K> for M
where
    M: Schema<PrimaryKey = K>,
    K: Default + Display + PartialEq,
{
}
