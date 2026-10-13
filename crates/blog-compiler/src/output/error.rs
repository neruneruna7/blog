use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("article slug cannot be used as an output directory: {0}")]
    InvalidArticleSlug(String),
    #[error("article bundle slug '{bundle_slug}' does not match metadata slug '{metadata_slug}'")]
    ArticleSlugMismatch {
        bundle_slug: String,
        metadata_slug: String,
    },
    #[error("duplicate article slug in output bundle: {0}")]
    DuplicateArticleSlug(String),
    #[error("asset path must be relative to public-root: '{0}'")]
    InvalidAssetPath(PathBuf),
    #[error("OGP asset '{path}' is missing from article '{slug}' assets")]
    MissingOgpAsset { slug: String, path: PathBuf },
    #[error("duplicate asset path in article '{slug}': '{path}'")]
    DuplicateAssetPath { slug: String, path: PathBuf },
    #[error("failed to create output directory '{path}': {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize JSON output '{path}': {source}")]
    SerializeJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write output file '{path}': {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
