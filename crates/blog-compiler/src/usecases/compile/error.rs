use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read article inputs: {0}")]
    Input(#[from] crate::input::error::Error),
    #[error("domain parsing failed: {0}")]
    DomainParse(#[from] crate::domain_parse::error::Error),
    #[error("article transformation failed: {0}")]
    ArticleTransform(#[from] crate::article_transform::error::Error),
}
