use std::collections::HashSet;

use serde::Serialize;

use crate::{
    commons::site::SiteTitle,
    features::article_parse::site::Site,
};

use super::{article::ArticleArtifact, error::ArtifactBuildError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfig {
    pub title: SiteTitle,
    pub description: String,
    pub language: String,
    // 現状特に使っていないのでコメントアウト
    // pub base_url: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteArtifacts {
    pub config: SiteConfig,
    pub articles: Vec<ArticleArtifact>,
}

impl SiteArtifacts {
    pub fn try_new(
        config: SiteConfig,
        articles: Vec<ArticleArtifact>,
    ) -> Result<Self, ArtifactBuildError> {
        let mut slugs = HashSet::new();
        for article in &articles {
            if !slugs.insert(article.slug.clone()) {
                return Err(ArtifactBuildError::DuplicateArticleSlug(
                    article.slug.as_str().to_owned(),
                ));
            }
        }

        Ok(Self { config, articles })
    }
}
