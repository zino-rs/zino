use crate::{error::Error, model::Model};

/// Hooks for the model.
pub trait ModelHooks: 'static + Send + Sync + Model {
    /// A hook running before scanning the table.
    #[inline]
    async fn before_scan(sql: &str) -> Result<(), Error> {
        println!("{sql}");
        Ok(())
    }

    /// A hook running after scanning the table.
    #[inline]
    async fn after_scan(num_rows: u64) -> Result<(), Error> {
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
        Ok(())
    }
}
