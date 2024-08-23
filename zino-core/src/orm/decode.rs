use super::{DatabaseDriver, DatabaseRow};
use crate::{error::Error, warn, BoxError, Decimal, Uuid};
use chrono::{DateTime, Local, NaiveDate, NaiveTime};
use sqlx::{Database, Decode, Row, Type};

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
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxError> {
        <NaiveDate as Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

impl<'r, DB> Decode<'r, DB> for crate::datetime::Time
where
    DB: Database,
    NaiveTime: Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxError> {
        <NaiveTime as Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

impl<'r, DB> Decode<'r, DB> for crate::datetime::DateTime
where
    DB: Database,
    DateTime<Local>: Decode<'r, DB>,
{
    #[inline]
    fn decode(value: <DB as Database>::ValueRef<'r>) -> Result<Self, BoxError> {
        <DateTime<Local> as Decode<'r, DB>>::decode(value).map(|dt| dt.into())
    }
}

/// Decodes a single value as `T` for the field in a row.
#[inline]
pub fn decode<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<T, Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    row.try_get_unchecked(field)
        .map_err(|err| warn!("fail to decode the `{}` field: {}", field, err))
}

/// Decodes a single value as `Decimal` for the field in a row.
#[cfg(any(
    feature = "orm-mariadb",
    feature = "orm-mysql",
    feature = "orm-postgres",
    feature = "orm-tidb"
))]
#[inline]
pub fn decode_decimal(row: &DatabaseRow, field: &str) -> Result<Decimal, Error> {
    row.try_get_unchecked(field).map_err(Error::from)
}

/// Decodes a single value as `Decimal` for the field in a row.
#[cfg(not(any(
    feature = "orm-mariadb",
    feature = "orm-mysql",
    feature = "orm-postgres",
    feature = "orm-tidb"
)))]
#[inline]
pub fn decode_decimal(row: &DatabaseRow, field: &str) -> Result<Decimal, Error> {
    decode::<String>(row, field)
        .and_then(|value| value.parse().map_err(Error::from))
        .map_err(Error::from)
}

/// Decodes a single value as `Uuid` for the field in a row.
#[cfg(feature = "orm-postgres")]
#[inline]
pub fn decode_uuid(row: &DatabaseRow, field: &str) -> Result<Uuid, Error> {
    row.try_get_unchecked(field).map_err(Error::from)
}

/// Decodes a single value as `Uuid` for the field in a row.
#[cfg(not(feature = "orm-postgres"))]
#[inline]
pub fn decode_uuid(row: &DatabaseRow, field: &str) -> Result<Uuid, Error> {
    decode::<String>(row, field)
        .and_then(|value| value.parse().map_err(Error::from))
        .map_err(Error::from)
}

/// Decodes a single value as `Vec<T>` for the field in a row.
#[cfg(feature = "orm-postgres")]
#[inline]
pub fn decode_array<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<Vec<T>, Error>
where
    T: for<'a> Decode<'a, DatabaseDriver> + sqlx::Type<DatabaseDriver>,
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
    <T as std::str::FromStr>::Err: std::error::Error + Send + 'static,
{
    use crate::{extension::JsonValueExt, JsonValue};

    let value = decode::<JsonValue>(row, field)?;
    if let Some(result) = value.parse_array() {
        result.map_err(Error::from)
    } else {
        Ok(Vec::new())
    }
}

/// Decodes a raw value at the index.
#[inline]
pub(super) fn decode_raw<'r, T>(
    field: &str,
    value: <DatabaseDriver as Database>::ValueRef<'r>,
) -> Result<T, sqlx::Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    T::decode(value).map_err(|source| {
        tracing::error!("fail to decode the `{}` field", field);
        sqlx::Error::ColumnDecode {
            index: field.to_owned(),
            source,
        }
    })
}
