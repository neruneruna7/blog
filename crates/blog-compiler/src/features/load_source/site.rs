use std::{fs, path::Path};

use serde::Deserialize;

use super::error::Error;

const SITE_CONFIG_NAME: &str = "site.toml";

#[derive(Debug, Clone, Deserialize)]
pub struct RawSiteConfig {
    pub title: String,
    pub description: String,
    pub language: String,
}

impl RawSiteConfig {
    pub(super) fn read(root: impl AsRef<Path>) -> Result<Self, Error> {
        let path = root.as_ref().join(SITE_CONFIG_NAME);
        let contents = fs::read_to_string(&path).map_err(|source| Error::ReadSiteConfig {
            path: path.clone(),
            source,
        })?;

        toml::from_str(&contents).map_err(|source| Error::ParseSiteConfig { path, source })
    }
}
