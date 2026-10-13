use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ArticleValidationError {
    #[error(transparent)]
    Value(#[from] crate::domain::ValueError),
    #[error("slug must be unique: {0}")]
    DuplicateSlug(String),
}

#[derive(Debug, Error)]
pub(crate) enum ArticleCompileError {
    #[error("failed to read article inputs: {0}")]
    Read(#[from] crate::input::error::Error),
    #[error("article validation failed: {0}")]
    Validation(#[from] ArticleValidationError),
    #[error("failed to compile article '{slug}': {source}")]
    TypstHtml {
        slug: String,
        #[source]
        source: typst_html_export::TypstHtmlExportError,
    },
}
