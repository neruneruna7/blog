use std::{collections::HashSet, path::Path};

use serde::Serialize;

use super::{article::ArticleBundle, create_directory, error::Error, write_json};

const SITE_CONFIG_FILE: &str = "site.json";
const ARTICLE_INDEX_FILE: &str = "article-index.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteConfig {
    pub title: String,
    pub description: String,
    pub language: String,
    pub base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteBundle {
    pub config: SiteConfig,
    pub articles: Vec<ArticleBundle>,
}

impl SiteBundle {
    pub fn write(&self, output_root: impl AsRef<Path>) -> Result<(), Error> {
        self.validate()?;

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

    fn validate(&self) -> Result<(), Error> {
        let mut slugs = HashSet::new();
        for article in &self.articles {
            article.validate()?;
            if !slugs.insert(article.slug.as_str()) {
                return Err(Error::DuplicateArticleSlug(article.slug.clone()));
            }
        }

        Ok(())
    }
}
