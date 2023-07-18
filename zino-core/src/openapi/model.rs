use crate::{
    datetime::{self, DateTime},
    extension::{JsonObjectExt, TomlTableExt},
    JsonValue, Map,
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
pub(crate) fn translate_model_entry(model: &mut Map, model_name: &str) {
    let Some(translation_keys) = MODEL_TRANSLATION_KEYS.get() else {
        return;
    };
    let Some(translations) = MODEL_TRANSLATIONS.get() else {
        return;
    };

    let mut data = Map::new();
    for (field, value) in model.iter() {
        let key = format!("{model_name}.{field}.translations");
        if !translation_keys.contains(&key.as_str()) {
            continue;
        }

        let Some(items) = translations.get(key.as_str()) else {
            continue;
        };
        if let Some(text_value) = translate_model_field(value, items) {
            let text_field = format!("{field}_text");
            data.upsert(text_field, text_value);
        }
    }
    model.append(&mut data);
}

/// Translates the model field.
fn translate_model_field(value: &JsonValue, items: &[(String, String)]) -> Option<JsonValue> {
    match value {
        JsonValue::String(s) => items.iter().find_map(|(k, v)| {
            if let Some(duration) = k.strip_prefix("$span:") {
                let Ok(duration) = datetime::parse_duration(duration) else {
                    return None;
                };
                let Ok(dt) = s.parse::<DateTime>() else {
                    return None;
                };
                (dt.span_between_now() <= duration).then_some(v.as_str().into())
            } else {
                (k == s).then_some(v.as_str().into())
            }
        }),
        JsonValue::Array(vec) => {
            let values = vec
                .iter()
                .map(|v| translate_model_field(v, items).unwrap_or_default())
                .collect::<Vec<_>>();
            Some(values.into())
        }
        _ => None,
    }
}

/// Model translations.
pub(super) static MODEL_TRANSLATIONS: OnceLock<HashMap<&str, Vec<(String, String)>>> =
    OnceLock::new();

/// Model translation keys.
pub(super) static MODEL_TRANSLATION_KEYS: OnceLock<Vec<&str>> = OnceLock::new();
