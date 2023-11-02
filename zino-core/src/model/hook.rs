use super::QueryContext;
use crate::{
    error::Error,
    model::{Model, Mutation, Query},
    Map,
};
use std::borrow::Cow;

/// Hooks for the model.
pub trait ModelHooks: Model {
    /// Model data.
    type Data: Default = ();
    /// Extension data.
    type Extension: Clone + Send + Sync + 'static = ();

    /// A hook running before extracting the model data.
    #[inline]
    async fn before_extract() -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after extracting the model data.
    #[inline]
    async fn after_extract(&mut self, _extension: Self::Extension) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running before validating the model data.
    #[inline]
    async fn before_validation(
        _model: &mut Map,
        _extension: Option<&Self::Extension>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after validating the model data.
    #[inline]
    async fn after_validation(&mut self, _model: &mut Map) -> Result<(), Error> {
        Ok(())
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
        let execution_time = ctx.start_time().elapsed();
        let execution_time_millis = execution_time.as_millis();
        if execution_time_millis > 3000 {
            tracing::warn!(
                query_id,
                query,
                arguments,
                execution_time_millis,
                "{message}"
            );
        } else if execution_time_millis > 1000 {
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
                execution_time_micros = execution_time.as_micros(),
                "{message}"
            );
        }
        Ok(())
    }

    /// A hook running before inserting a model into the table.
    #[inline]
    async fn before_insert(&mut self) -> Result<Self::Data, Error> {
        self.before_upsert().await
    }

    /// A hook running after inserting a model into the table.
    #[inline]
    async fn after_insert(ctx: &QueryContext, data: Self::Data) -> Result<(), Error> {
        Self::after_upsert(ctx, data).await?;
        ctx.emit_metrics("insert");
        Ok(())
    }

    /// A hook running before logically deleting a model from the table.
    #[inline]
    async fn before_soft_delete(&mut self) -> Result<Self::Data, Error> {
        self.before_upsert().await
    }

    /// A hook running after logically deleting a model from the table.
    #[inline]
    async fn after_soft_delete(ctx: &QueryContext, data: Self::Data) -> Result<(), Error> {
        Self::after_upsert(ctx, data).await?;
        ctx.emit_metrics("soft_delete");
        Ok(())
    }

    /// A hook running before locking a model in the table.
    #[inline]
    async fn before_lock(&mut self) -> Result<Self::Data, Error> {
        self.before_upsert().await
    }

    /// A hook running after locking a model in the table.
    #[inline]
    async fn after_lock(ctx: &QueryContext, data: Self::Data) -> Result<(), Error> {
        Self::after_upsert(ctx, data).await?;
        ctx.emit_metrics("lock");
        Ok(())
    }

    /// A hook running before updating a model in the table.
    #[inline]
    async fn before_update(&mut self) -> Result<Self::Data, Error> {
        self.before_upsert().await
    }

    /// A hook running after updating a model in the table.
    #[inline]
    async fn after_update(ctx: &QueryContext, data: Self::Data) -> Result<(), Error> {
        Self::after_upsert(ctx, data).await?;
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

    /// A hook running before counting the models in the table.
    #[inline]
    async fn before_count(_query: &Query) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after counting the models in the table.
    #[inline]
    async fn after_count(ctx: &QueryContext) -> Result<(), Error> {
        if !ctx.is_success() {
            ctx.record_error("fail to count the models in the table");
        }
        ctx.emit_metrics("count");
        Ok(())
    }

    /// A hook running before selecting the models with a `Query` from the table.
    #[inline]
    async fn before_query(_query: &Query) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after selecting the models with a `Query` from the table.
    #[inline]
    async fn after_query(ctx: &QueryContext) -> Result<(), Error> {
        if !ctx.is_success() {
            ctx.record_error("fail to select the models from the table");
        }
        ctx.emit_metrics("query");
        Ok(())
    }

    /// A hook running before updating the models with a `Mutation` in the table.
    #[inline]
    async fn before_mutation(_query: &Query, _mutation: &mut Mutation) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after updating the models with a `Mutation` in the table.
    #[inline]
    async fn after_mutation(ctx: &QueryContext) -> Result<(), Error> {
        if !ctx.is_success() {
            ctx.record_error("fail to update the models in the table");
        }
        ctx.emit_metrics("mutation");
        Ok(())
    }

    /// A hook running before listing the models with a `Query` from the table.
    #[inline]
    async fn before_list(
        _query: &mut Query,
        _extension: Option<&Self::Extension>,
    ) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after decoding the model as a `Map`.
    #[inline]
    async fn after_decode(_model: &mut Map) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running before returning the model data as a HTTP response.
    #[inline]
    async fn before_respond(
        _model: &mut Map,
        _extension: Option<&Self::Extension>,
    ) -> Result<(), Error> {
        Ok(())
    }
}
