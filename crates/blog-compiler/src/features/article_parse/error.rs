use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Value(#[from] crate::commons::ValueError),
    #[error("article slug must be unique: {0}")]
    DuplicateArticleSlug(String),
}
