use std::{collections::HashSet, path::Path};

use serde::Serialize;

use crate::domain::site::SiteTitle;

use super::{
    article::ArticleArtifact,
    create_directory,
    error::{ArtifactBuildError, ArtifactWriteError},
    write_json,
};

const SITE_CONFIG_FILE: &str = "site.json";
const ARTICLE_INDEX_FILE: &str = "article-index.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfig {
    pub title: SiteTitle,
    pub description: String,
    pub language: String,
    // 現状特に使っていないのでコメントアウト
    // pub base_url: String,
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

    pub fn write(&self, output_root: impl AsRef<Path>) -> Result<(), ArtifactWriteError> {
        let output_root = output_root.as_ref();
        create_directory(output_root)?;
        write_json(&output_root.join(SITE_CONFIG_FILE), &self.config)?;

        let article_index = self
            .articles
            .iter()
            .map(|article| &article.metadata)
            .collect::<Vec<_>>();
        write_json(&output_root.join(ARTICLE_INDEX_FILE), &article_index)?;

        for article in &self.articles {
            article.write(output_root)?;
        }

        Ok(())
    }
}
