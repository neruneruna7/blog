use crate::domain_parse::article::Article;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledArticle {
    pub article: Article,
    pub html: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessedArticle {
    pub article: Article,
    pub html: String,
    pub ogp_image: Vec<u8>,
    pub warnings: Vec<ArticleCompileWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleCompileWarning {
    pub url: String,
    pub reason: String,
}
