use crate::{extension::JsonObjectExt, model::Translation, Map};
use std::{collections::HashMap, sync::OnceLock};

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

        let Some(translation) = translations.get(key.as_str()) else {
            continue;
        };
        if let Some(text_value) = translation.translate(value) {
            let text_field = format!("{field}_text");
            data.upsert(text_field, text_value);
        }
    }
    model.append(&mut data);
}

/// Model translations.
pub(super) static MODEL_TRANSLATIONS: OnceLock<HashMap<&str, Translation>> = OnceLock::new();

/// Model translation keys.
pub(super) static MODEL_TRANSLATION_KEYS: OnceLock<Vec<&str>> = OnceLock::new();
