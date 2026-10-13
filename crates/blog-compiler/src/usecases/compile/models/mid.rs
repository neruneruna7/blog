use std::path::PathBuf;

use crate::domain::article::{
    ArticleId, ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedArticle {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArticle {
    pub article: ValidatedArticle,
    pub html: String,
}
