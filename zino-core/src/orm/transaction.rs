use super::{
    executor::Executor, mutation::MutationExt, query::QueryExt, schema::Schema, DatabaseDriver,
};
use crate::{
    error::Error,
    extension::JsonValueExt,
    model::{EncodeColumn, Mutation, Query},
    BoxFuture, Map,
};
use std::fmt::Display;

#[cfg(feature = "orm-sqlx")]
use sqlx::Acquire;

/// An in-progress database transaction.
pub trait Transaction<K, Tx>: Schema<PrimaryKey = K>
where
    K: Default + Display + PartialEq,
{
    /// Executes the specific operations inside of a transaction.
    /// If the operations return an error, the transaction will be rolled back;
    /// if not, the transaction will be committed.
    async fn transaction<F, T>(tx: F) -> Result<T, Error>
    where
        F: for<'t> FnOnce(&'t mut Tx) -> BoxFuture<'t, Result<T, Error>>;

    /// Executes the queries sequentially inside of a transaction.
    /// If it returns an error, the transaction will be rolled back;
    /// if not, the transaction will be committed.
    async fn transactional_execute(queries: &[&str], params: Option<&Map>) -> Result<u64, Error>;

    /// Inserts the model and its associations inside of a transaction.
    async fn transactional_insert<M: Schema>(self, models: Vec<M>) -> Result<u64, Error>;

    /// Updates the models inside of a transaction.
    async fn transactional_update<M: Schema>(
        queries: (&Query, &Query),
        mutations: (&mut Mutation, &mut Mutation),
    ) -> Result<u64, Error>;

    /// Deletes the models inside of a transaction.
    async fn transactional_delete<M: Schema>(queries: (&Query, &Query)) -> Result<u64, Error>;
}

#[cfg(feature = "orm-sqlx")]
impl<'c, M, K> Transaction<K, sqlx::Transaction<'c, DatabaseDriver>> for M
where
    M: Schema<PrimaryKey = K>,
    K: Default + Display + PartialEq,
{
    async fn transaction<F, T>(tx: F) -> Result<T, Error>
    where
        F: for<'t> FnOnce(
            &'t mut sqlx::Transaction<'c, DatabaseDriver>,
        ) -> BoxFuture<'t, Result<T, Error>>,
    {
        let mut transaction = Self::acquire_writer().await?.pool().begin().await?;
        let data = tx(&mut transaction).await?;
        transaction.commit().await?;
        Ok(data)
    }

    async fn transactional_execute(queries: &[&str], params: Option<&Map>) -> Result<u64, Error> {
        let mut transaction = Self::acquire_writer().await?.pool().begin().await?;
        let connection = transaction.acquire().await?;

        let mut total_rows = 0;
        for query in queries {
            let (sql, values) = Query::prepare_query(query, params);

            let mut ctx = Self::before_scan(&sql).await?;
            let mut arguments = values
                .iter()
                .map(|v| v.to_string_unquoted())
                .collect::<Vec<_>>();

            let rows_affected = connection
                .execute_with(&sql, &arguments)
                .await?
                .rows_affected();
            total_rows += rows_affected;
            ctx.set_query(sql);
            ctx.append_arguments(&mut arguments);
            ctx.set_query_result(Some(rows_affected), true);
            Self::after_scan(&ctx).await?;
        }
        transaction.commit().await?;
        Ok(total_rows)
    }

    async fn transactional_insert<S: Schema>(mut self, models: Vec<S>) -> Result<u64, Error> {
        let mut transaction = Self::acquire_writer().await?.pool().begin().await?;
        let connection = transaction.acquire().await?;

        // Inserts the model
        let model_data = self.before_insert().await?;
        let map = self.into_map();
        let table_name = Self::table_name();
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

        let mut total_rows = 0;
        let query_result = connection.execute(&sql).await?;
        let (last_insert_id, rows_affected) = Query::parse_query_result(query_result);
        let success = rows_affected == 1;
        if let Some(last_insert_id) = last_insert_id {
            ctx.set_last_insert_id(last_insert_id);
        }
        total_rows += rows_affected;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), success);
        Self::after_scan(&ctx).await?;
        Self::after_insert(&ctx, model_data).await?;

        // Inserts associations
        let columns = S::columns();
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

        let table_name = S::table_name();
        let fields = S::fields().join(", ");
        let values = values.join(", ");
        let sql = format!("INSERT INTO {table_name} ({fields}) VALUES {values};");
        let mut ctx = S::before_scan(&sql).await?;

        let rows_affected = connection.execute(&sql).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        S::after_scan(&ctx).await?;

        // Commits the transaction
        transaction.commit().await?;
        Ok(total_rows)
    }

    async fn transactional_update<S: Schema>(
        queries: (&Query, &Query),
        mutations: (&mut Mutation, &mut Mutation),
    ) -> Result<u64, Error> {
        let mut transaction = Self::acquire_writer().await?.pool().begin().await?;
        let connection = transaction.acquire().await?;

        let query = queries.0;
        let mutation = mutations.0;
        Self::before_mutation(query, mutation).await?;

        let table_name = Self::table_name();
        let filters = query.format_filters::<Self>();
        let updates = mutation.format_updates::<Self>();
        let sql = format!("UPDATE {table_name} SET {updates} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;

        let mut total_rows = 0;
        let rows_affected = connection.execute(&sql).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        Self::after_scan(&ctx).await?;
        Self::after_mutation(&ctx).await?;

        let query = queries.1;
        let mutation = mutations.1;
        S::before_mutation(query, mutation).await?;

        let table_name = S::table_name();
        let filters = query.format_filters::<S>();
        let updates = mutation.format_updates::<S>();
        let sql = format!("UPDATE {table_name} SET {updates} {filters};");
        let mut ctx = S::before_scan(&sql).await?;

        let rows_affected = connection.execute(&sql).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        S::after_scan(&ctx).await?;
        S::after_mutation(&ctx).await?;

        // Commits the transaction
        transaction.commit().await?;
        Ok(total_rows)
    }

    async fn transactional_delete<S: Schema>(queries: (&Query, &Query)) -> Result<u64, Error> {
        let mut transaction = Self::acquire_writer().await?.pool().begin().await?;
        let connection = transaction.acquire().await?;

        let query = queries.0;
        Self::before_query(query).await?;

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let sql = format!("DELETE FROM {table_name} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;

        let mut total_rows = 0;
        let rows_affected = connection.execute(&sql).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;

        let query = queries.1;
        S::before_query(query).await?;

        let table_name = query.format_table_name::<S>();
        let filters = query.format_filters::<S>();
        let sql = format!("DELETE FROM {table_name} {filters};");
        let mut ctx = S::before_scan(&sql).await?;

        let rows_affected = connection.execute(&sql).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query(sql);
        ctx.set_query_result(Some(rows_affected), true);
        S::after_scan(&ctx).await?;
        S::after_query(&ctx).await?;

        // Commits the transaction
        transaction.commit().await?;
        Ok(total_rows)
    }
}
