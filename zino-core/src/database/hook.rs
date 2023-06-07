use super::QueryContext;
use crate::{error::Error, model::Model};
use std::borrow::Cow;

/// Hooks for the model.
pub trait ModelHooks: 'static + Send + Sync + Model {
    /// A hook running before scanning the table.
    #[inline]
    async fn before_scan(sql: &str) -> Result<QueryContext<'_>, Error> {
        tracing::debug!(sql);
        Ok(QueryContext::new(sql))
    }

    /// A hook running after scanning the table.
    #[inline]
    async fn after_scan<'a>(ctx: &QueryContext<'a>, num_rows: u64) -> Result<(), Error> {
        let execution_time = ctx.start_time().elapsed().as_millis();
        let message = match num_rows {
            0 => Cow::Borrowed("no rows affected or fetched"),
            1 => Cow::Borrowed("one row affected or fetched"),
            _ => Cow::Owned(format!("{num_rows} rows affected or fetched")),
        };
        tracing::debug!(execution_time, "{message}");
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
            tracing::error!(sql = ctx.sql(), "fail to insert a model into the table");
        }
        Ok(())
    }

    /// A hook running before deleting a model in the table.
    #[inline]
    async fn before_delete(&self) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after deleting a model in the table.
    #[inline]
    async fn after_delete<'a>(self, ctx: &QueryContext<'a>, success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!(sql = ctx.sql(), "fail to detele a model in the table");
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
            tracing::error!(sql = ctx.sql(), "fail to update a model into the table");
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
            tracing::error!(sql = ctx.sql(), "fail to upsert a model into the table");
        }
        Ok(())
    }
}
