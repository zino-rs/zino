use super::{DatabaseDriver, DatabaseRow};
use crate::{error::Error, BoxError};
use chrono::{DateTime, Local};
use sqlx::{database::HasValueRef, Database, Decode, Row};

impl<'r, DB> Decode<'r, DB> for crate::datetime::DateTime
where
    DB: Database,
    DateTime<Local>: Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxError> {
        <DateTime<Local> as Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

/// Decodes a single value for the field in a row.
#[inline]
pub fn decode<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<T, Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    row.try_get_unchecked(field).map_err(Error::from)
}
