use crate::{extension::JsonObjectExt, model::Translation, Map};
use convert_case::{Case, Casing};
use std::{collections::HashMap, sync::LazyLock};

/// Translates the model data.
pub(crate) fn translate_model_entry(model: &mut Map, model_name: &str) {
    let mut data = Map::new();
    let model_name_prefix = format!("{model_name}.");
    for (key, translation) in MODEL_TRANSLATIONS.iter() {
        if let Some(field) = key.strip_prefix(&model_name_prefix)
            && let Some(value) = model.get(field)
        {
            let text_field = format!("{field}_text");
            let text_value = translation.translate(value).unwrap_or_else(|| value.clone());
            data.upsert(text_field, text_value);
        }
    }
    model.append(&mut data);
}

/// Model translations.
static MODEL_TRANSLATIONS: LazyLock<HashMap<&str, Translation>> = LazyLock::new(|| {
    let mut model_translations = HashMap::new();
    if let Some(definitions) = super::MODEL_DEFINITIONS.get() {
        for (model_name, fields) in definitions.iter() {
            for (field, value) in fields {
                let translation = value.as_table().map(Translation::with_config);
                if let Some(translation) = translation && translation.is_ready() {
                    let model_name = model_name.to_case(Case::Snake);
                    let model_key = format!("{model_name}.{field}").leak() as &'static str;
                    model_translations.insert(model_key, translation);
                }
            }
        }
    }
    model_translations
});
