use std::path::PathBuf;

use chrono::{DateTime, FixedOffset};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedArticle {
    pub root: PathBuf,
    pub entry_point: PathBuf,
    pub id: ArticleId,
    pub slug: Slug,
    pub title: Title,
    pub description: Description,
    pub tags: Vec<Tag>,
    pub published_at: PublishedAt,
    pub updated_at: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArticle {
    pub article: ValidatedArticle,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleId(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Title(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Description(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublishedAt(pub DateTime<FixedOffset>);
