use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to inspect article path '{path}': {source}")]
    InspectArticlePath {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to serialize article configuration: {0}")]
    SerializeArticleConfig(#[source] toml::ser::Error),
    #[error("failed to create directory '{path}': {source}")]
    CreateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to write article file '{path}': {source}")]
    WriteFile {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("article directory already exists: '{path}'")]
    ArticlePathExists { path: PathBuf },
    #[error("failed to create article directory '{path}': {source}")]
    CommitArticle {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(
        "could not create an article staging directory after {attempts} attempts under '{path}'"
    )]
    TemporaryDirectoryExhausted { path: PathBuf, attempts: usize },
}
