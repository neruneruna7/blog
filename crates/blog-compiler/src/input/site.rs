use std::{fs, path::Path};

use serde::Deserialize;

use super::{article::RawArticle, error::Error};

const SITE_CONFIG_NAME: &str = "site.toml";

#[derive(Debug, Clone)]
pub struct RawSiteBundle {
    pub articles: Vec<RawArticle>,
    pub site_config: RawSiteConfig,
}

impl RawSiteBundle {
    /// ファイルシステムなどから，記事データやサイト設定などを読み出す．
    ///
    /// ## example
    /// ```
    /// let input_bundle = RawSiteBundle::read().unwrap();
    ///
    /// let articles = input_bundle.articles;
    /// let site_config = input_bundle.site_config;
    /// ```
    pub fn read() -> Result<Self, Error> {
        let root = super::TYP_COMPILE_ROOT;
        let articles = RawArticle::read_all(root)?;
        let site_config = RawSiteConfig::read(root)?;

        Ok(Self {
            articles,
            site_config,
        })
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawSiteConfig {
    pub title: String,
    pub description: String,
    pub language: String,
}

impl RawSiteConfig {
    fn read(root: impl AsRef<Path>) -> Result<Self, Error> {
        let path = root.as_ref().join(SITE_CONFIG_NAME);
        let contents = fs::read_to_string(&path).map_err(|source| Error::ReadSiteConfig {
            path: path.clone(),
            source,
        })?;

        toml::from_str(&contents).map_err(|source| Error::ParseSiteConfig { path, source })
    }
}
