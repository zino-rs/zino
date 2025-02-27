use super::{DatabaseDriver, DatabaseRow};
use sqlx::{Database, Decode, Row, ValueRef};
use zino_core::{Decimal, Uuid, error::Error, warn};

/// Decodes a single value as `T` for the field in a row.
#[inline]
pub fn decode<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<T, Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    row.try_get_unchecked(field)
        .map_err(|err| warn!("fail to decode the `{}` field: {}", field, err))
}

/// Decodes a single value as `T` for the field in a row,
/// returning `None` if it was not found.
#[inline]
pub fn decode_optional<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<Option<T>, Error>
where
    T: Decode<'r, DatabaseDriver>,
{
    match row.try_get_raw(field) {
        Ok(value) => {
            if value.is_null() {
                Ok(None)
            } else {
                let value = decode_raw(field, value)?;
                Ok(Some(value))
            }
        }
        Err(err) => {
            if let sqlx::Error::ColumnNotFound(_) = err {
                Ok(None)
            } else {
                Err(warn!("fail to get the `{}` field: {}", field, err))
            }
        }
    }
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
    match row.try_get_raw(field) {
        Ok(value) => {
            if value.is_null() {
                warn!("decode the NULL as a zero value");
                Ok(Decimal::ZERO)
            } else {
                let value = decode_raw(field, value)?;
                Ok(value)
            }
        }
        Err(err) => {
            if let sqlx::Error::ColumnNotFound(_) = err {
                Ok(Decimal::ZERO)
            } else {
                Err(warn!("fail to get the `{}` field: {}", field, err))
            }
        }
    }
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
    let Some(value) = decode_optional::<String>(row, field)? else {
        return Ok(Decimal::ZERO);
    };
    value
        .parse()
        .map_err(|err| warn!("fail to decode the `{}` field: {}", field, err))
}

/// Decodes a single value as `Uuid` for the field in a row.
#[cfg(feature = "orm-postgres")]
#[inline]
pub fn decode_uuid(row: &DatabaseRow, field: &str) -> Result<Uuid, Error> {
    match row.try_get_raw(field) {
        Ok(value) => {
            if value.is_null() {
                Ok(Uuid::nil())
            } else {
                let id = decode_raw(field, value)?;
                Ok(id)
            }
        }
        Err(err) => {
            if let sqlx::Error::ColumnNotFound(_) = err {
                Ok(Uuid::nil())
            } else {
                Err(warn!("fail to get the `{}` field: {}", field, err))
            }
        }
    }
}

/// Decodes a single value as `Uuid` for the field in a row.
#[cfg(not(feature = "orm-postgres"))]
#[inline]
pub fn decode_uuid(row: &DatabaseRow, field: &str) -> Result<Uuid, Error> {
    let Some(value) = decode_optional::<String>(row, field)? else {
        return Ok(Uuid::nil());
    };
    value
        .parse()
        .map_err(|err| warn!("fail to decode the `{}` field: {}", field, err))
}

/// Decodes a single value as `Vec<T>` for the field in a row.
#[cfg(feature = "orm-postgres")]
#[inline]
pub fn decode_array<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<Vec<T>, Error>
where
    T: for<'a> Decode<'a, DatabaseDriver> + sqlx::Type<DatabaseDriver>,
{
    match row.try_get_raw(field) {
        Ok(value) => {
            if value.is_null() {
                warn!("decode the NULL as an empty array");
                Ok(Vec::new())
            } else {
                let vec = decode_raw(field, value)?;
                Ok(vec)
            }
        }
        Err(err) => {
            if let sqlx::Error::ColumnNotFound(_) = err {
                Ok(Vec::new())
            } else {
                Err(warn!("fail to get the `{}` field: {}", field, err))
            }
        }
    }
}

/// Decodes a single value as `Vec<T>` for the field in a row.
#[cfg(feature = "orm-mariadb")]
#[inline]
pub fn decode_array<'r, T>(row: &'r DatabaseRow, field: &str) -> Result<Vec<T>, Error>
where
    T: Decode<'r, DatabaseDriver> + serde::de::DeserializeOwned,
{
    let Some(value) = decode_optional::<String>(row, field)? else {
        return Ok(Vec::new());
    };
    if value.starts_with('[') && value.ends_with(']') {
        serde_json::from_str(&value)
            .map_err(|err| warn!("fail to decode the `{}` field: {}", field, err))
    } else {
        zino_core::bail!("invalid array data for the `{}` field", field);
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
    use zino_core::{JsonValue, extension::JsonValueExt};

    let Some(value) = decode_optional::<JsonValue>(row, field)? else {
        return Ok(Vec::new());
    };
    if let Some(result) = value.parse_array() {
        result.map_err(|err| warn!("fail to decode the `{}` field: {}", field, err))
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
