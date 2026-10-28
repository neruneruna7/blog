use std::{fs, path::Path};

use super::{error::Error, toml_config};

const SITE_CONFIG_NAME: &str = "site.toml";

#[derive(Debug, Clone)]
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

        Self::parse(&contents).map_err(|source| Error::ParseSiteConfig { path, source })
    }

    fn parse(contents: &str) -> Result<Self, toml_config::ConfigError> {
        let table = toml_config::parse(contents)?;

        Ok(Self {
            title: toml_config::required_string(&table, "title")?,
            description: toml_config::required_string(&table, "description")?,
            language: toml_config::required_string(&table, "language")?,
        })
    }
}
