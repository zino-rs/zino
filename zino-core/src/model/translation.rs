use crate::{
    datetime::{self, DateTime},
    extension::TomlTableExt,
    JsonValue,
};
use toml::Table;

/// Model field translations.
#[derive(Debug, Clone)]
pub struct Translation {
    /// Mappings.
    mappings: Vec<(String, String)>,
}

impl Translation {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            mappings: Vec::new(),
        }
    }

    /// Creates a new instance with the configuration.
    pub fn with_config(config: &Table) -> Self {
        let Some(translations) = config.get_array("translations") else {
            return Self::default();
        };
        let mappings = translations
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
            .collect::<Vec<_>>();
        Self { mappings }
    }

    /// Translates the value.
    pub fn translate(&self, value: &JsonValue) -> Option<JsonValue> {
        match value {
            JsonValue::String(s) => self.mappings.iter().find_map(|(k, v)| {
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
                    .map(|v| self.translate(v).unwrap_or_default())
                    .collect::<Vec<_>>();
                Some(values.into())
            }
            _ => None,
        }
    }

    /// Returns `true` if the translation is ready for use.
    #[inline]
    pub fn is_ready(&self) -> bool {
        !self.mappings.is_empty()
    }
}

impl Default for Translation {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
