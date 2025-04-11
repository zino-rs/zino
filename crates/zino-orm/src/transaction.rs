use super::{
    DatabaseDriver, EncodeColumn, executor::Executor, mutation::MutationExt, query::QueryExt,
    schema::Schema,
};
use std::fmt::Display;
use zino_core::{
    BoxFuture, Map,
    error::Error,
    extension::JsonValueExt,
    model::{Mutation, Query},
};

#[cfg(feature = "orm-sqlx")]
use sqlx::Acquire;

/// An in-progress database transaction.
///
/// # Examples
///
/// ```rust,ignore
/// use crate::model::{Account, AccountColumn, Order, Stock, StockColumn};
/// use zino_orm::{MutationBuilder, QueryBuilder, Schema, Transaction};
///
/// let user_id = "0193d8e6-2970-7b52-bc06-80a981212aa9";
/// let product_id = "0193c06d-bee6-7070-a5e7-9659161bddb5";
///
/// let order = Order::from_customer(user_id, product_id);
/// let quantity = order.quantity();
/// let total_price = order.total_price();
/// let order_ctx = order.prepare_insert()?;
///
/// let stock_query = QueryBuilder::new()
///     .and_eq(StockColumn::ProductId, product_id)
///     .and_ge(StockColumn::Quantity, quantity)
///     .build();
/// let mut stock_mutation = MutationBuilder::new()
///     .inc(StockColumn::Quantity, -quantity)
///     .build();
/// let stock_ctx = Stock::prepare_update_one(&stock_query, &mut stock_mutation).await?;
///
/// let account_query = QueryBuilder::new()
///     .and_eq(AccountColumn::UserId, user_id)
///     .and_ge(AccountColumn::Balance, total_price)
///     .build();
/// let mut account_mutation = MutationBuilder::new()
///     .inc(AccountColumn::Balance, -total_price)
///     .build();
/// let account_ctx = Account::prepare_update_one(&account_query, &mut account_mutation).await?;
///
/// Order::transaction(move |tx| Box::pin(async move {
///      let connection = tx.acquire().await?;
///      connection.execute(order_ctx.query()).await?;
///      connection.execute(stock_ctx.query()).await?;
///      connection.execute(account_ctx.query()).await?;
///      Ok(())
/// })).await?;
/// ```
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
            ctx.set_query(sql);

            let mut arguments = values
                .iter()
                .map(|v| v.to_string_unquoted())
                .collect::<Vec<_>>();
            let rows_affected = connection
                .execute_with(ctx.query(), &arguments)
                .await?
                .rows_affected();
            total_rows += rows_affected;
            ctx.append_arguments(&mut arguments);
            ctx.set_query_result(rows_affected, true);
            Self::after_scan(&ctx).await?;
        }
        transaction.commit().await?;
        Ok(total_rows)
    }

    async fn transactional_insert<S: Schema>(mut self, associations: Vec<S>) -> Result<u64, Error> {
        let mut transaction = Self::acquire_writer().await?.pool().begin().await?;
        let connection = transaction.acquire().await?;

        // Inserts the model
        let model_data = self.before_insert().await?;
        let table_name = if let Some(table) = self.before_prepare().await? {
            Query::escape_table_name(&table)
        } else {
            Query::escape_table_name(Self::table_name())
        };
        let map = self.into_map();
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

        let mut total_rows = 0;
        let query_result = connection.execute(ctx.query()).await?;
        let (last_insert_id, rows_affected) = Query::parse_query_result(query_result);
        let success = rows_affected == 1;
        if let Some(last_insert_id) = last_insert_id {
            ctx.set_last_insert_id(last_insert_id);
        }
        total_rows += rows_affected;
        ctx.set_query_result(rows_affected, success);
        Self::after_scan(&ctx).await?;
        Self::after_insert(&ctx, model_data).await?;

        // Inserts associations
        let columns = S::columns();
        let mut values = Vec::with_capacity(associations.len());
        for mut association in associations.into_iter() {
            let _association_data = association.before_insert().await?;
            let map = association.into_map();
            let entries = columns
                .iter()
                .map(|col| col.encode_value(map.get(col.name())))
                .collect::<Vec<_>>()
                .join(", ");
            values.push(format!("({entries})"));
        }

        let table_name = Query::escape_table_name(S::table_name());
        let fields = S::fields().join(", ");
        let values = values.join(", ");
        let sql = format!("INSERT INTO {table_name} ({fields}) VALUES {values};");
        let mut ctx = S::before_scan(&sql).await?;
        ctx.set_query(sql);

        let rows_affected = connection.execute(ctx.query()).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query_result(rows_affected, true);
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

        let table_name = query.format_table_name::<Self>();
        let filters = query.format_filters::<Self>();
        let updates = mutation.format_updates::<Self>();
        let sql = format!("UPDATE {table_name} SET {updates} {filters};");
        let mut ctx = Self::before_scan(&sql).await?;
        ctx.set_query(sql);

        let mut total_rows = 0;
        let rows_affected = connection.execute(ctx.query()).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query_result(rows_affected, true);
        Self::after_scan(&ctx).await?;
        Self::after_mutation(&ctx).await?;

        let query = queries.1;
        let mutation = mutations.1;
        S::before_mutation(query, mutation).await?;

        let table_name = query.format_table_name::<S>();
        let filters = query.format_filters::<S>();
        let updates = mutation.format_updates::<S>();
        let sql = format!("UPDATE {table_name} SET {updates} {filters};");
        let mut ctx = S::before_scan(&sql).await?;
        ctx.set_query(sql);

        let rows_affected = connection.execute(ctx.query()).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query_result(rows_affected, true);
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
        ctx.set_query(sql);

        let mut total_rows = 0;
        let rows_affected = connection.execute(ctx.query()).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query_result(rows_affected, true);
        Self::after_scan(&ctx).await?;
        Self::after_query(&ctx).await?;

        let query = queries.1;
        S::before_query(query).await?;

        let table_name = query.format_table_name::<S>();
        let filters = query.format_filters::<S>();
        let sql = format!("DELETE FROM {table_name} {filters};");
        let mut ctx = S::before_scan(&sql).await?;
        ctx.set_query(sql);

        let rows_affected = connection.execute(ctx.query()).await?.rows_affected();
        total_rows += rows_affected;
        ctx.set_query_result(rows_affected, true);
        S::after_scan(&ctx).await?;
        S::after_query(&ctx).await?;

        // Commits the transaction
        transaction.commit().await?;
        Ok(total_rows)
    }
}
