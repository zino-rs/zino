use super::{Entity, QueryBuilder, Schema};
use http::Uri;
use std::{borrow::Cow, net::IpAddr, path::Path};
use url::Url;
use zino_core::{
    Decimal, JsonValue, Map, Uuid,
    datetime::{Date, DateTime, Time},
    extension::JsonObjectExt,
};

/// A generic interface for converting into SQL values.
pub trait IntoSqlValue {
    /// Converts `self` to a SQL value.
    fn into_sql_value(self) -> JsonValue;
}

macro_rules! impl_into_sql_value {
    ($($Ty: ty),+ $(,)?) => {
        $(
            impl IntoSqlValue for $Ty {
                #[inline]
                fn into_sql_value(self) -> JsonValue {
                    self.into()
                }
            }
        )+
    }
}

impl_into_sql_value!(
    (),
    bool,
    u8,
    u16,
    u32,
    u64,
    usize,
    i8,
    i16,
    i32,
    i64,
    isize,
    f32,
    f64,
    &str,
    String,
    JsonValue,
    Map,
    Date,
    Time,
);

impl IntoSqlValue for DateTime {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        if cfg!(feature = "orm-postgres") {
            self.to_string().into()
        } else {
            self.to_utc_timestamp().into()
        }
    }
}

impl IntoSqlValue for Decimal {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.to_string().into()
    }
}

impl IntoSqlValue for IpAddr {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.to_string().into()
    }
}

impl IntoSqlValue for Uuid {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.to_string().into()
    }
}

impl IntoSqlValue for &Path {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.to_str().into()
    }
}

impl IntoSqlValue for &Uri {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.to_string().into()
    }
}

impl IntoSqlValue for &Url {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.as_str().into()
    }
}

impl IntoSqlValue for Cow<'_, str> {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.into()
    }
}

impl<T: IntoSqlValue> IntoSqlValue for Option<T> {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        self.map(|v| v.into_sql_value()).into()
    }
}

impl<T: IntoSqlValue> IntoSqlValue for Vec<T> {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        JsonValue::Array(self.into_iter().map(|v| v.into_sql_value()).collect())
    }
}

impl<T: IntoSqlValue, const N: usize> IntoSqlValue for [T; N] {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        JsonValue::Array(self.into_iter().map(|v| v.into_sql_value()).collect())
    }
}

impl<T: Clone + IntoSqlValue> IntoSqlValue for &[T] {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        JsonValue::Array(self.iter().map(|v| v.to_owned().into_sql_value()).collect())
    }
}

impl<E: Entity + Schema> IntoSqlValue for QueryBuilder<E> {
    #[inline]
    fn into_sql_value(self) -> JsonValue {
        Map::from_entry("$subquery", self.build_subquery()).into()
    }
}
