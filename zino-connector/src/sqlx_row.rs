use serde::ser::{self, Serialize, SerializeMap, Serializer};
use sqlx::{Column, ColumnIndex, Database, Decode, Row, TypeInfo, ValueRef};
use std::borrow::Cow;

/// A generic struct for the row.
pub(super) struct SerializeRow<R: Row>(pub(super) R);

impl<'r, R: Row> Serialize for &'r SerializeRow<R>
where
    usize: ColumnIndex<R>,
    Cow<'r, str>: Decode<'r, <R as Row>::Database>,
    f64: Decode<'r, <R as Row>::Database>,
    i64: Decode<'r, <R as Row>::Database>,
    bool: Decode<'r, <R as Row>::Database>,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let columns = self.0.columns();
        let mut map = serializer.serialize_map(Some(columns.len()))?;
        for col in columns {
            let key = col.name();
            match self.0.try_get_raw(col.ordinal()) {
                Ok(raw_value) if !raw_value.is_null() => match raw_value.type_info().name() {
                    "BOOL" | "BOOLEAN" => map_serialize::<_, _, bool>(&mut map, key, raw_value)?,
                    "DOUBLE" | "DOUBLE PRECISION" | "FLOAT" | "FLOAT4" | "FLOAT8" | "NUMERIC"
                    | "REAL" => map_serialize::<_, _, f64>(&mut map, key, raw_value)?,
                    "BIGINT" | "BIGINT UNSIGNED" | "BIGSERIAL" | "INT" | "INT2" | "INT4"
                    | "INT8" | "INTEGER" | "INT UNSIGNED" | "SERIAL" | "SMALLINT"
                    | "SMALLINT UNSIGNED" | "SMALLSERIAL" | "TINYINT" | "TINYINT UNSIGNED" => {
                        map_serialize::<_, _, i64>(&mut map, key, raw_value)?
                    }
                    // Deserialize as a string by default
                    _ => map_serialize::<_, _, Cow<'_, str>>(&mut map, key, raw_value)?,
                },
                _ => map.serialize_entry(key, &())?, // Serialize null
            }
        }
        map.end()
    }
}

fn map_serialize<'r, M: SerializeMap, DB: Database, T: Decode<'r, DB> + Serialize>(
    map: &mut M,
    key: &str,
    raw_value: <DB as Database>::ValueRef<'r>,
) -> Result<(), M::Error> {
    let value = T::decode(raw_value).map_err(ser::Error::custom)?;
    map.serialize_entry(key, &value)
}
