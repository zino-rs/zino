use super::QueryContext;
use crate::{
    error::Error,
    model::{Model, Query},
    Map,
};
use std::borrow::Cow;

/// Hooks for the model.
pub trait ModelHooks: Model {
    /// Associated data.
    type Data: Default = ();

    /// A hook running before validating the model data.
    #[inline]
    async fn before_validation(data: Map) -> Result<Map, Error> {
        Ok(data)
    }

    /// A hook running after validating the model data.
    #[inline]
    async fn after_validation(data: Map) -> Result<Map, Error> {
        Ok(data)
    }

    /// A hook running before creating the table.
    #[inline]
    async fn before_create_table() -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after creating the table.
    #[inline]
    async fn after_create_table() -> Result<(), Error> {
        Ok(())
    }

    /// A hook running before scanning the table.
    #[inline]
    async fn before_scan(query: &str) -> Result<QueryContext, Error> {
        let ctx = QueryContext::new();
        let query_id = ctx.query_id().to_string();
        tracing::debug!(query_id, query);
        Ok(ctx)
    }

    /// A hook running after scanning the table.
    async fn after_scan(ctx: &QueryContext) -> Result<(), Error> {
        let query_id = ctx.query_id().to_string();
        let query = ctx.query();
        let arguments = ctx.format_arguments();
        let message = match ctx.rows_affected() {
            Some(0) => Cow::Borrowed("no rows affected or fetched"),
            Some(1) => Cow::Borrowed("only one row affected or fetched"),
            Some(num_rows) if num_rows > 1 => {
                Cow::Owned(format!("{num_rows} rows affected or fetched"))
            }
            _ => Cow::Borrowed("the query result has not been recorded"),
        };
        let execution_time_millis = ctx.start_time().elapsed().as_millis();
        if execution_time_millis > 1000 {
            tracing::warn!(
                query_id,
                query,
                arguments,
                execution_time_millis,
                "{message}"
            );
        } else if execution_time_millis > 100 {
            tracing::info!(
                query_id,
                query,
                arguments,
                execution_time_millis,
                "{message}"
            );
        } else {
            tracing::debug!(
                query_id,
                query,
                arguments,
                execution_time_millis,
                "{message}"
            );
        }
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
            ctx.record_error("fail to insert a model into the table");
        }
        ctx.emit_metrics("insert");
        Ok(())
    }

    /// A hook running before deleting a model from the table.
    #[inline]
    async fn before_delete(&mut self) -> Result<Self::Data, Error> {
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
        ctx.emit_metrics("delete");
        Ok(())
    }

    /// A hook running before logically deleting a model from the table.
    #[inline]
    async fn before_soft_delete(&mut self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after logically deleting a model from the table.
    #[inline]
    async fn after_soft_delete(ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        if !ctx.is_success() {
            ctx.record_error("fail to logically delete a model from the table");
        }
        ctx.emit_metrics("soft_delete");
        Ok(())
    }

    /// A hook running before locking a model in the table.
    #[inline]
    async fn before_lock(&mut self) -> Result<Self::Data, Error> {
        Ok(Self::Data::default())
    }

    /// A hook running after locking a model in the table.
    #[inline]
    async fn after_lock(ctx: &QueryContext, _data: Self::Data) -> Result<(), Error> {
        if !ctx.is_success() {
            ctx.record_error("fail to lock a model in the table");
        }
        ctx.emit_metrics("lock");
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
            ctx.record_error("fail to update a model into the table");
        }
        ctx.emit_metrics("update");
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
            ctx.record_error("fail to upsert a model into the table");
        }
        ctx.emit_metrics("upsert");
        Ok(())
    }

    /// A hook running before selecting the models from the table.
    #[inline]
    async fn before_select(_query: &Query) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after selecting the models from the table.
    #[inline]
    async fn after_select(ctx: &QueryContext) -> Result<(), Error> {
        if !ctx.is_success() {
            ctx.record_error("fail to select the models from the table");
        }
        ctx.emit_metrics("select");
        Ok(())
    }
}
