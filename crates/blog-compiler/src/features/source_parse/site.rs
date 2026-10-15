use crate::{commons::site::SiteTitle, features::load_source::site::RawSiteConfig};

use super::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Site {
    pub title: SiteTitle,
    pub description: String,
    pub language: String,
}

impl Site {
    pub fn parse(raw_site_config: RawSiteConfig) -> Result<Self, Error> {
        Ok(Self {
            title: SiteTitle::new(raw_site_config.title)?,
            description: raw_site_config.description,
            language: raw_site_config.language,
        })
    }
}
