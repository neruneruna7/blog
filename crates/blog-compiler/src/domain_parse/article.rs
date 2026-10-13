use std::{collections::HashMap, path::PathBuf};

use crate::{
    domain::article::{ArticleId, ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt},
    input::article::RawArticle,
};

use super::error::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub root: PathBuf,
    pub entry_point: PathBuf,
    pub id: ArticleId,
    pub slug: Slug,
    pub title: ArticleTitle,
    pub description: Description,
    pub tags: Vec<Tag>,
    pub published_at: PublishedAt,
    pub updated_at: UpdatedAt,
}

impl Article {
    pub fn parse(raw_article: RawArticle) -> Result<Self, Error> {
        let RawArticle {
            root,
            entry_point,
            config,
        } = raw_article;

        Ok(Self {
            root,
            entry_point,
            id: ArticleId::new(config.id)?,
            slug: Slug::new(config.slug)?,
            title: ArticleTitle::new(config.title)?,
            description: Description::new(config.description)?,
            tags: config.tags.into_iter().map(Tag::new).collect(),
            published_at: PublishedAt::new(config.published_at)?,
            updated_at: UpdatedAt::new(config.updated_at)?,
        })
    }

    pub fn parse_all(
        raw_articles: impl IntoIterator<Item = RawArticle>,
    ) -> Result<HashMap<Slug, Self>, Error> {
        let mut articles = HashMap::new();
        for raw_article in raw_articles {
            let article = Self::parse(raw_article)?;
            let slug = article.slug.clone();
            match articles.entry(slug.clone()) {
                std::collections::hash_map::Entry::Occupied(_) => {
                    return Err(Error::DuplicateArticleSlug(slug.as_str().to_owned()));
                }
                std::collections::hash_map::Entry::Vacant(entry) => {
                    entry.insert(article);
                }
            }
        }

        Ok(articles)
    }
}
