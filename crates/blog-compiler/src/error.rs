use std::path::PathBuf;

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
    #[error("failed to compile article '{slug}': {source}")]
    TypstHtml {
        slug: String,
        #[source]
        source: typst_html_export::TypstHtmlExportError,
    },
    #[error(transparent)]
    Postprocess(#[from] PostprocessError),
}

#[derive(Debug, Error)]
pub(crate) enum PostprocessError {
    #[error("article {slug}: {processor} postprocessor failed: {reason}")]
    Processor {
        slug: String,
        processor: &'static str,
        reason: String,
    },
}

#[derive(Debug, Error)]
pub(crate) enum ArticleWriteError {
    #[error("article output root must have a parent directory: {0}")]
    MissingOutputParent(PathBuf),
    #[error("failed to serialize metadata for article '{slug}': {source}")]
    SerializeMetadata {
        slug: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to create output directory '{path}': {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write output file '{path}': {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}

#[derive(Debug, Error)]
pub(crate) enum OgpSiteConfigError {
    #[error("failed to read site configuration '{path}': {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse site configuration '{path}': {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("site title must not be empty in '{path}'")]
    EmptyTitle { path: PathBuf },
}
