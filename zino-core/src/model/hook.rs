use super::QueryContext;
use crate::{error::Error, model::Model};
use std::borrow::Cow;

/// Hooks for the model.
pub trait ModelHooks: Model {
    /// A hook running before scanning the table.
    #[inline]
    async fn before_scan(query: &str) -> Result<QueryContext<'_>, Error> {
        tracing::debug!(query);
        Ok(QueryContext::new(query))
    }

    /// A hook running after scanning the table.
    #[inline]
    async fn after_scan<'a>(ctx: &QueryContext<'a>, num_rows: u64) -> Result<(), Error> {
        let message = match num_rows {
            0 => Cow::Borrowed("no rows affected or fetched"),
            1 => Cow::Borrowed("one row affected or fetched"),
            _ => Cow::Owned(format!("{num_rows} rows affected or fetched")),
        };
        let duration = ctx.start_time().elapsed();
        let execution_time = duration.as_millis();
        if execution_time > 1000 {
            tracing::warn!(execution_time, "{message}");
        } else if execution_time > 100 {
            tracing::info!(execution_time, "{message}");
        } else {
            tracing::debug!(execution_time, "{message}");
        }
        metrics::histogram!("zino_model_query_duration_seconds", duration.as_secs_f64());
        Ok(())
    }

    /// A hook running before inserting a model into the table.
    #[inline]
    async fn before_insert(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after inserting a model into the table.
    #[inline]
    async fn after_insert<'a>(ctx: &QueryContext<'a>, success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!(query = ctx.query(), "fail to insert a model into the table");
        }
        Ok(())
    }

    /// A hook running before deleting a model from the table.
    #[inline]
    async fn before_delete(&self) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after deleting a model from the table.
    #[inline]
    async fn after_delete<'a>(self, ctx: &QueryContext<'a>, success: bool) -> Result<(), Error> {
        let query = ctx.query();
        if success {
            tracing::warn!(query, "a model was deleted from the table");
        } else {
            tracing::error!(query, "fail to detele a model from the table");
        }
        Ok(())
    }

    /// A hook running before updating a model into the table.
    #[inline]
    async fn before_update(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after updating a model into the table.
    #[inline]
    async fn after_update<'a>(ctx: &QueryContext<'a>, success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!(query = ctx.query(), "fail to update a model into the table");
        }
        Ok(())
    }

    /// A hook running before updating or inserting a model into the table.
    #[inline]
    async fn before_upsert(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after updating or inserting a model into the table.
    #[inline]
    async fn after_upsert<'a>(ctx: &QueryContext<'a>, success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!(query = ctx.query(), "fail to upsert a model into the table");
        }
        Ok(())
    }
}
