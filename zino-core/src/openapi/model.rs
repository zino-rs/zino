use crate::{
    extension::{JsonObjectExt, TomlTableExt},
    Map,
};
use std::{collections::HashMap, sync::OnceLock};
use toml::Table;

/// Parses field translations.
pub(super) fn parse_field_translations(config: &Table) -> Vec<(String, String)> {
    let Some(translations) = config.get_array("translations") else {
        return Vec::new();
    };
    translations
        .iter()
        .filter_map(|v| v.as_array())
        .filter_map(|v| {
            if let [v0, v1, ..] = v.as_slice() {
                v0.as_str()
                    .zip(v1.as_str())
                    .map(|(s0, s1)| (s0.to_owned(), s1.to_owned()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

/// Translates the model data.
pub(crate) fn translate_model_entry(model: &mut Map, key: &str) {
    let Some(translations) = MODEL_TRANSLATIONS.get() else {
        return;
    };
    if translations.is_empty() {
        return;
    }

    let mut data = Map::new();
    for (field, value) in model.iter() {
        if let Some(s) = value.as_str() {
            let model_field = format!("{key}.{field}.translations");
            if let Some(vec) = translations.get(model_field.as_str()) &&
                let Some(value) = vec.iter().find_map(|v| (v.0 == s).then_some(v.1.as_str()))
            {
                let field = format!("{field}_text");
                data.upsert(field, value);
            }
        }
    }
    model.append(&mut data);
}

/// Model translations.
pub(super) static MODEL_TRANSLATIONS: OnceLock<HashMap<String, Vec<(String, String)>>> =
    OnceLock::new();
