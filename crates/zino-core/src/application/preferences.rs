use ini::{Ini, Properties};
use std::{path::PathBuf, str::FromStr};

/// Application preferences from an `INI` file.
#[derive(Debug, Clone, Default)]
pub struct Preferences {
    /// The file path.
    path: PathBuf,
    /// The file content.
    content: Ini,
}

impl Preferences {
    /// Creates a new instance.
    #[inline]
    pub fn new(path: PathBuf) -> Self {
        let content = Ini::load_from_file(&path).unwrap_or_default();
        Self { path, content }
    }

    /// Returns a reference to the specific section.
    #[inline]
    pub fn section(&self, section: &str) -> Option<&Properties> {
        self.content.section(Some(section))
    }

    /// Extracts a value for the key in a specific section.
    #[inline]
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.content.get_from(Some(section), key)
    }

    /// Extracts a value for the key in a specific section
    /// and parses it as an instance of `T`.
    #[inline]
    pub fn parse<T: FromStr>(
        &self,
        section: &str,
        key: &str,
    ) -> Option<Result<T, <T as FromStr>::Err>> {
        self.get(section, key).map(|s| s.parse())
    }

    /// Updates the value for the key in a specific section.
    pub fn update(&mut self, section: &str, key: &str, value: String) {
        let path = self.path.as_path();
        self.content.with_section(Some(section)).set(key, value);
        if let Err(err) = self.content.write_to_file(path) {
            tracing::error!("fail to write `{}`: {}", path.display(), err);
        }
    }
}
