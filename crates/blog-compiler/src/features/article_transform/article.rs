use crate::commons::article::{ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformedArticle {
    pub slug: Slug,
    pub title: ArticleTitle,
    pub description: Description,
    pub tags: Vec<Tag>,
    pub published_at: PublishedAt,
    pub updated_at: UpdatedAt,
    pub html: String,
    pub ogp_image: Vec<u8>,
    pub warnings: Vec<ArticleCompileWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleCompileWarning {
    pub url: String,
    pub reason: String,
}
