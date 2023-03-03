//! Application specific models.
use crate::{request::Validation, Map};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Error, Value};

mod column;
mod mutation;
mod query;
mod row;

pub use column::{Column, EncodeColumn};
pub use mutation::Mutation;
pub use query::Query;
pub use row::DecodeRow;

/// A type for data model.
pub trait Model: Default + Serialize + DeserializeOwned {
    /// Creates a new instance.
    fn new() -> Self;

    /// Updates the model using the json object and returns the validation result.
    #[must_use]
    fn read_map(&mut self, data: Map) -> Validation;

    /// Attempts to construct a model from a json object.
    #[inline]
    fn try_from_map(data: Map) -> Result<Self, Error> {
        serde_json::from_value(Value::from(data))
    }

    /// Consumes the model and returns as a json object.
    #[must_use]
    fn into_map(self) -> Map {
        match serde_json::to_value(self) {
            Ok(Value::Object(map)) => map,
            _ => panic!("the model cann't be converted to a json object"),
        }
    }
}
