use super::{schema::Schema, DatabaseDriver};
use crate::{error::Error, BoxFuture};
use std::fmt::Display;

/// An in-progress database transaction.
pub trait Transaction<K, Tx>: Schema<PrimaryKey = K>
where
    K: Default + Display + PartialEq,
{
    /// Executes the specific operations inside a transaction.
    /// If the operations return an error, the transaction will be rolled back;
    /// if not, the transaction will be committed.
    async fn transaction<F, T>(tx: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(&'a Tx) -> BoxFuture<'a, Result<T, Error>>;
}

#[cfg(feature = "orm-sqlx")]
impl<'c, K, M> Transaction<K, sqlx::Transaction<'c, DatabaseDriver>> for M
where
    K: Default + Display + PartialEq,
    M: Schema<PrimaryKey = K>,
{
    /// Executes the specific operations inside a transaction.
    /// If the operations return an error, the transaction will be rolled back;
    /// if not, the transaction will be committed.
    async fn transaction<F, T>(tx: F) -> Result<T, Error>
    where
        F: for<'a> FnOnce(
            &'a sqlx::Transaction<'c, DatabaseDriver>,
        ) -> BoxFuture<'a, Result<T, Error>>,
    {
        let pool = Self::acquire_writer().await?.pool();
        let transaction = pool.begin().await?;
        let data = tx(&transaction).await?;
        transaction.commit().await?;
        Ok(data)
    }
}
