use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read article inputs: {0}")]
    Input(#[from] crate::features::input::error::Error),
    #[error("domain parsing failed: {0}")]
    DomainParse(#[from] crate::features::article_parse::error::Error),
    #[error("article transformation failed: {0}")]
    ArticleTransform(#[from] crate::features::article_transform::error::Error),
}
