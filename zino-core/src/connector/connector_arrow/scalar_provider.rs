use super::ScalarValueExt;
use crate::Map;
use datafusion::{
    arrow::datatypes::DataType, error::DataFusionError, scalar::ScalarValue, variable::VarProvider,
};
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};
use toml::Table;

/// A provider for scalar values.
#[derive(Debug, Clone)]
pub(super) struct ScalarValueProvider(HashMap<String, ScalarValue>);

impl ScalarValueProvider {
    /// Creates a new instance.
    #[inline]
    pub(super) fn new() -> Self {
        Self(HashMap::new())
    }

    /// Reads scalar values from a TOML table.
    pub(super) fn read_toml_table(&mut self, table: &Table) {
        for (key, value) in table {
            let key = key.replace('-', "_");
            let value = ScalarValue::from_toml_value(value.to_owned());
            self.insert(key, value);
        }
    }

    /// Reads scalar values from a JSON object.
    pub(super) fn read_json_object(&mut self, map: &Map) {
        for (key, value) in map {
            let key = key.replace('-', "_");
            let value = ScalarValue::from_json_value(value.to_owned());
            self.insert(key, value);
        }
    }
}

impl Default for ScalarValueProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for ScalarValueProvider {
    type Target = HashMap<String, ScalarValue>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ScalarValueProvider {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl VarProvider for ScalarValueProvider {
    fn get_value(&self, var_names: Vec<String>) -> Result<ScalarValue, DataFusionError> {
        var_names
            .iter()
            .find_map(|name| self.get(name.trim_start_matches('@')))
            .map(|value| value.to_owned())
            .ok_or_else(|| DataFusionError::Plan(format!("fail to get variable `{var_names:?}`")))
    }

    fn get_type(&self, var_names: &[String]) -> Option<DataType> {
        var_names.iter().find_map(|name| {
            self.get(name.trim_start_matches('@'))
                .map(|value| value.get_datatype())
        })
    }
}
