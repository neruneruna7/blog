use serde::Serialize;

use crate::{commons::site::SiteTitle, features::source_parse::site::Site};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfig {
    pub title: SiteTitle,
    pub description: String,
    pub language: String,
}

impl From<&Site> for SiteConfig {
    fn from(site: &Site) -> Self {
        Self {
            title: site.title.clone(),
            description: site.description.clone(),
            language: site.language.clone(),
        }
    }
}
