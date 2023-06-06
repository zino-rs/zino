use crate::{error::Error, model::Model};
use std::borrow::Cow;

/// Hooks for the model.
pub trait ModelHooks: 'static + Send + Sync + Model {
    /// A hook running before scanning the table.
    #[inline]
    async fn before_scan(sql: &str) -> Result<(), Error> {
        tracing::debug!(sql);
        Ok(())
    }

    /// A hook running after scanning the table.
    #[inline]
    async fn after_scan(num_rows: u64) -> Result<(), Error> {
        let message = match num_rows {
            0 => Cow::Borrowed("no rows affected or fetched"),
            1 => Cow::Borrowed("one row affected or fetched"),
            _ => Cow::Owned(format!("{num_rows} rows affected or fetched")),
        };
        tracing::debug!("{message}");
        Ok(())
    }

    /// A hook running before inserting a model into the table.
    #[inline]
    async fn before_insert(&mut self) -> Result<(), Error> {
        Ok(())
    }

    /// A hook running after inserting a model into the table.
    #[inline]
    async fn after_insert(success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!("fail to insert a model into the table");
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
    async fn after_delete(self, success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!("fail to detele a model in the table");
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
    async fn after_update(success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!("fail to update a model into the table");
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
    async fn after_upsert(success: bool) -> Result<(), Error> {
        if !success {
            tracing::error!("fail to upsert a model into the table");
        }
        Ok(())
    }
}
