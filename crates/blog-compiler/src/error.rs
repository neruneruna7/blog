use thiserror::Error;

#[derive(Debug, Error)]
pub(crate) enum ArticleValidationError {
    #[error("{field} must not be empty")]
    EmptyValue { field: &'static str },
    #[error("{field} must be at most {maximum} characters (got {actual})")]
    TextTooLong {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },
    #[error(
        "slug must contain lowercase ASCII letters and digits separated by single hyphens: {0}"
    )]
    InvalidSlug(String),
    #[error("slug must be unique: {0}")]
    DuplicateSlug(String),
    #[error("{field} must be RFC3339: {source}")]
    InvalidDateTime {
        field: &'static str,
        #[source]
        source: chrono::ParseError,
    },
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
