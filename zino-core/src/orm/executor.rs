use crate::error::Error;

/// Executing queries against the database.
pub trait Executor {
    /// A type for the database row.
    type Row;

    /// A type for the query result.
    type QueryResult;

    /// Executes the query and return the total number of rows affected.
    async fn execute(&self, sql: &str) -> Result<Self::QueryResult, Error>;

    /// Executes the query with arguments and return the total number of rows affected.
    async fn execute_with<T: ToString>(
        &self,
        sql: &str,
        arguments: &[T],
    ) -> Result<Self::QueryResult, Error>;

    /// Executes the query and return all the generated results.
    async fn fetch(&self, sql: &str) -> Result<Vec<Self::Row>, Error>;

    /// Executes the query with arguments and return all the generated results.
    async fn fetch_with<T: ToString>(
        &self,
        sql: &str,
        arguments: &[T],
    ) -> Result<Vec<Self::Row>, Error>;

    /// Executes the query and returns exactly one row.
    async fn fetch_one(&self, sql: &str) -> Result<Self::Row, Error>;

    /// Executes the query and returns at most one row.
    async fn fetch_optional(&self, sql: &str) -> Result<Option<Self::Row>, Error>;

    /// Executes the query with arguments and returns at most one row.
    async fn fetch_optional_with<T: ToString>(
        &self,
        sql: &str,
        arguments: &[T],
    ) -> Result<Option<Self::Row>, Error>;
}

#[cfg(feature = "orm-sqlx")]
impl Executor for sqlx::Pool<super::DatabaseDriver> {
    type Row = super::DatabaseRow;
    type QueryResult = <super::DatabaseDriver as sqlx::Database>::QueryResult;

    async fn execute(&self, sql: &str) -> Result<Self::QueryResult, Error> {
        sqlx::query(sql).execute(self).await.map_err(Error::from)
    }

    async fn execute_with<T: ToString>(
        &self,
        sql: &str,
        arguments: &[T],
    ) -> Result<Self::QueryResult, Error> {
        let mut query = sqlx::query(sql);
        for arg in arguments {
            query = query.bind(arg.to_string());
        }
        query.execute(self).await.map_err(Error::from)
    }

    async fn fetch(&self, sql: &str) -> Result<Vec<Self::Row>, Error> {
        use futures::TryStreamExt;
        use std::sync::atomic::Ordering::Relaxed;

        let mut stream = sqlx::query(sql).fetch(self);
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        let mut rows = Vec::with_capacity(stream.size_hint().0.min(max_rows));
        while let Some(row) = stream.try_next().await?
            && max_rows > 0
        {
            rows.push(row);
            max_rows -= 1;
        }
        Ok(rows)
    }

    async fn fetch_with<T: ToString>(
        &self,
        sql: &str,
        arguments: &[T],
    ) -> Result<Vec<Self::Row>, Error> {
        use futures::TryStreamExt;
        use std::sync::atomic::Ordering::Relaxed;

        let mut query = sqlx::query(sql);
        for arg in arguments {
            query = query.bind(arg.to_string());
        }

        let mut stream = query.fetch(self);
        let mut max_rows = super::MAX_ROWS.load(Relaxed);
        let mut rows = Vec::with_capacity(stream.size_hint().0.min(max_rows));
        while let Some(row) = stream.try_next().await?
            && max_rows > 0
        {
            rows.push(row);
            max_rows -= 1;
        }
        Ok(rows)
    }

    async fn fetch_one(&self, sql: &str) -> Result<Self::Row, Error> {
        sqlx::query(sql).fetch_one(self).await.map_err(Error::from)
    }

    async fn fetch_optional(&self, sql: &str) -> Result<Option<Self::Row>, Error> {
        sqlx::query(sql)
            .fetch_optional(self)
            .await
            .map_err(Error::from)
    }

    async fn fetch_optional_with<T: ToString>(
        &self,
        sql: &str,
        arguments: &[T],
    ) -> Result<Option<Self::Row>, Error> {
        let mut query = sqlx::query(sql);
        for arg in arguments {
            query = query.bind(arg.to_string());
        }
        query.fetch_optional(self).await.map_err(Error::from)
    }
}
