use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArtifactBuildError {
    #[error(
        "article artifact slug '{artifact_slug}' does not match metadata slug '{metadata_slug}'"
    )]
    ArticleSlugMismatch {
        artifact_slug: String,
        metadata_slug: String,
    },
    #[error("duplicate article slug in site artifacts: {0}")]
    DuplicateArticleSlug(String),
    #[error("asset path must be relative to public-root: '{0}'")]
    InvalidAssetPath(PathBuf),
    #[error("OGP asset '{path}' is missing from article '{slug}' assets")]
    MissingOgpAsset { slug: String, path: PathBuf },
    #[error("duplicate asset path in article '{slug}': '{path}'")]
    DuplicateAssetPath { slug: String, path: PathBuf },
}

#[derive(Debug, Error)]
pub enum ArtifactWriteError {
    #[error("failed to create artifact directory '{path}': {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize JSON artifact '{path}': {source}")]
    SerializeJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write artifact file '{path}': {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
