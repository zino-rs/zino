use super::QueryContext;
use crate::{error::Error, model::Model};
use std::borrow::Cow;

/// Hooks for the model.
pub trait ModelHooks: Model {
    /// Associated data.
    type Data: Default = ();

    /// A hook running before scanning the table.
    #[inline]
    async fn before_scan(query: &str) -> Result<QueryContext, Error> {
        let ctx = QueryContext::new();
        let query_id = ctx.query_id().to_string();
        tracing::debug!(query, query_id);
        Ok(ctx)
    }

    /// A hook running after scanning the table.
    #[inline]
    async fn after_scan(ctx: &QueryContext) -> Result<(), Error> {
        let query = ctx.query();
        let query_id = ctx.query_id().to_string();
        let message = match ctx.rows_affected() {
            Some(0) => Cow::Borrowed("no rows affected or fetched"),
            Some(1) => Cow::Borrowed("only one row affected or fetched"),
            Some(num_rows) if num_rows > 1 => {
                Cow::Owned(format!("{num_rows} rows affected or fetched"))
            }
            _ => Cow::Borrowed("the query result has not been recorded"),
        };
        let duration = ctx.start_time().elapsed();
        let execution_time = duration.as_millis();
        if execution_time > 1000 {
            tracing::warn!(query, query_id, execution_time, "{message}");
        } else if execution_time > 100 {
            tracing::info!(query, query_id, execution_time, "{message}");
        } else {
            tracing::debug!(query, query_id, execution_time, "{message}");
        }
        metrics::histogram!("zino_model_query_duration_seconds", duration.as_secs_f64());
        Ok(())
    }

    /// A hook running before inserting a model into the table.
    #[inline]
    async fn before_insert(&mut self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after inserting a model into the table.
    #[inline]
    async fn after_insert(ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        if !ctx.is_success() {
            let query = ctx.query();
            let query_id = ctx.query_id().to_string();
            tracing::error!(query, query_id, "fail to insert a model into the table");
        }
        Ok(())
    }

    /// A hook running before deleting a model from the table.
    #[inline]
    async fn before_delete(&self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after deleting a model from the table.
    #[inline]
    async fn after_delete(self, ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        let query = ctx.query();
        let query_id = ctx.query_id().to_string();
        if ctx.is_success() {
            tracing::warn!(query, query_id, "a model was deleted from the table");
        } else {
            tracing::error!(query, query_id, "fail to detele a model from the table");
        }
        Ok(())
    }

    /// A hook running before logically deleting a model into the table.
    #[inline]
    async fn before_soft_delete(&mut self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after logically deleting a model into the table.
    #[inline]
    async fn after_soft_delete(ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        if !ctx.is_success() {
            let query = ctx.query();
            let query_id = ctx.query_id().to_string();
            tracing::error!(
                query,
                query_id,
                "fail to logically delete a model into the table"
            );
        }
        Ok(())
    }

    /// A hook running before updating a model into the table.
    #[inline]
    async fn before_update(&mut self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after updating a model into the table.
    #[inline]
    async fn after_update(ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        if !ctx.is_success() {
            let query = ctx.query();
            let query_id = ctx.query_id().to_string();
            tracing::error!(query, query_id, "fail to update a model into the table");
        }
        Ok(())
    }

    /// A hook running before updating or inserting a model into the table.
    #[inline]
    async fn before_upsert(&mut self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after updating or inserting a model into the table.
    #[inline]
    async fn after_upsert(ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        if !ctx.is_success() {
            let query = ctx.query();
            let query_id = ctx.query_id().to_string();
            tracing::error!(query, query_id, "fail to upsert a model into the table");
        }
        Ok(())
    }
}
