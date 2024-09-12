use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ZinoToml {
    pub(crate) remote: Remote,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Remote {
    pub(crate) name: String,
    pub(crate) branch: String,
}

impl Default for Remote {
    fn default() -> Self {
        Self {
            name: "origin".to_string(),
            branch: "main".to_string(),
        }
    }
}
