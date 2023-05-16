use crate::BoxError;
use chrono::{DateTime, Local};
use sqlx::{database::HasValueRef, Database, Decode};

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
