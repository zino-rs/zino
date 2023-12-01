use super::{DatabaseDriver, DatabaseRow};
use crate::{error::Error, BoxError};
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use sqlx::{database::HasValueRef, Database, Decode, Row, Type};

impl<DB> Type<DB> for crate::datetime::Date
where
    DB: Database,
    NaiveDate: Type<DB>,
{
    #[inline]
    fn type_info() -> <DB as Database>::TypeInfo {
        <NaiveDate as Type<DB>>::type_info()
    }
}

impl<DB> Type<DB> for crate::datetime::Time
where
    DB: Database,
    NaiveTime: Type<DB>,
{
    #[inline]
    fn type_info() -> <DB as Database>::TypeInfo {
        <NaiveTime as Type<DB>>::type_info()
    }
}

impl<DB> Type<DB> for crate::datetime::DateTime
where
    DB: Database,
    DateTime<Local>: Type<DB>,
{
    #[inline]
    fn type_info() -> <DB as Database>::TypeInfo {
        <DateTime<Local> as Type<DB>>::type_info()
    }
}

impl<'r, DB> Decode<'r, DB> for crate::datetime::Date
where
    DB: Database,
    NaiveDate: Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxError> {
        <NaiveDate as Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

impl<'r, DB> Decode<'r, DB> for crate::datetime::Time
where
    DB: Database,
    NaiveTime: Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxError> {
        <NaiveTime as Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

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

/// Decodes a single value as `T` for the field in a row.
#[inline]
pub fn decode<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<T, Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    row.try_get_unchecked(field).map_err(Error::from)
}

/// Decodes a single value as `Vec<T>` for the field in a row.
#[cfg(feature = "orm-postgres")]
#[inline]
pub fn decode_array<'r, T>(row: &'r sqlx::postgres::PgRow, field: &str) -> Result<Vec<T>, Error>
where
    T: for<'a> Decode<'a, sqlx::Postgres> + sqlx::Type<sqlx::Postgres>,
{
    row.try_get_unchecked(field).map_err(Error::from)
}

/// Decodes a single value as `Vec<T>` for the field in a row.
#[cfg(feature = "orm-mariadb")]
#[inline]
pub fn decode_array<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<Vec<T>, Error>
where
    T: Decode<'r, DatabaseDriver> + serde::de::DeserializeOwned,
{
    let value = decode::<String>(row, field)?;
    if value.starts_with('[') && value.ends_with(']') {
        serde_json::from_str(&value).map_err(Error::from)
    } else {
        crate::bail!("invalid array data for the `{}` field", field);
    }
}

/// Decodes a single value as `Vec<T>` for the field in a row.
#[cfg(not(any(feature = "orm-mariadb", feature = "orm-postgres")))]
#[inline]
pub fn decode_array<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<Vec<T>, Error>
where
    T: Decode<'r, DatabaseDriver> + std::str::FromStr,
{
    use crate::{extension::JsonValueExt, JsonValue};

    decode::<JsonValue>(row, field).map(|value| value.parse_array().unwrap_or_default())
}

/// Decodes a raw value at the index.
#[inline]
pub(super) fn decode_raw<'r, T>(
    field: &str,
    value: <DatabaseDriver as HasValueRef<'r>>::ValueRef,
) -> Result<T, sqlx::Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    T::decode(value).map_err(|source| sqlx::Error::ColumnDecode {
        index: field.to_owned(),
        source,
    })
}
