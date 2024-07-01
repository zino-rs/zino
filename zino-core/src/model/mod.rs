//! Domain specific models.
use crate::{validation::Validation, AvroValue, JsonValue, Map, Record};
use serde::{de::DeserializeOwned, Serialize};

mod column;
mod context;
mod hook;
mod mutation;
mod query;
mod reference;
mod row;
mod translation;

#[doc(no_inline)]
pub use apache_avro::schema;

pub use column::{Column, EncodeColumn};
pub use context::QueryContext;
pub use hook::ModelHooks;
pub use mutation::Mutation;
pub use query::Query;
pub use reference::Reference;
pub use row::DecodeRow;
pub use translation::Translation;

/// General data model.
///
/// This trait can be derived by `zino_derive::Model`.
pub trait Model: Default + Serialize + DeserializeOwned {
    /// Model name.
    const MODEL_NAME: &'static str;
    /// Data item name.
    const ITEM_NAME: (&'static str, &'static str) = ("entry", "entries");

    /// Creates a new instance.
    #[inline]
    fn new() -> Self {
        Self::default()
    }

    /// Returns the model name.
    #[inline]
    fn model_name() -> &'static str {
        Self::MODEL_NAME
    }

    /// Updates the model using the json object and returns the validation result.
    #[must_use]
    fn read_map(&mut self, data: &Map) -> Validation {
        let mut validation = Validation::new();
        if data.is_empty() {
            let message = format!("the `{}` model data should be nonempty", Self::MODEL_NAME);
            validation.record("data", message);
        }
        validation
    }

    /// Attempts to construct a model from a json object.
    #[inline]
    fn try_from_map(data: Map) -> Result<Self, serde_json::Error> {
        serde_json::from_value(JsonValue::from(data))
    }

    /// Attempts to construct a model from an Avro record.
    #[inline]
    fn try_from_avro_record(data: Record) -> Result<Self, apache_avro::Error> {
        apache_avro::from_value(&AvroValue::Record(data))
    }

    /// Consumes the model and returns as a json object.
    ///
    /// # Panics
    ///
    /// It will panic if the model cann't be converted to a json object.
    #[must_use]
    fn into_map(self) -> Map {
        match serde_json::to_value(self) {
            Ok(JsonValue::Object(map)) => map,
            _ => panic!(
                "the `{}` model cann't be converted to a json object",
                Self::MODEL_NAME
            ),
        }
    }

    /// Consumes the model and returns as an Avro record.
    ///
    /// # Panics
    ///
    /// It will panic if the model cann't be converted to an Avro record.
    #[must_use]
    fn into_avro_record(self) -> Record {
        match apache_avro::to_value(self) {
            Ok(AvroValue::Record(record)) => record,
            _ => panic!(
                "the `{}` model cann't be converted to an Avro record",
                Self::MODEL_NAME
            ),
        }
    }

    /// Constructs an instance of `Map` for the data item.
    #[inline]
    fn data_item(value: impl Into<JsonValue>) -> Map {
        let item_name = Self::ITEM_NAME.0;
        let mut map = Map::new();
        map.insert(item_name.to_owned(), value.into());
        map
    }

    /// Constructs an instance of `Map` for the data items.
    #[inline]
    fn data_items<T: Into<JsonValue>>(values: Vec<T>) -> Map {
        let item_name = Self::ITEM_NAME.1;
        let mut map = Map::new();
        map.insert(["num", item_name].join("_"), values.len().into());
        map.insert(item_name.to_owned(), values.into());
        map
    }
}
