use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to serialize metadata JSON '{path}': {source}")]
    SerializeJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to write metadata JSON '{path}': {source}")]
    WriteJson {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect generated article directory '{path}': {source}")]
    ReadArticleDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect generated article '{path}': {source}")]
    InspectArticle {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to remove obsolete article metadata '{path}': {source}")]
    RemoveObsoleteMetadata {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to remove empty article directory '{path}': {source}")]
    RemoveEmptyDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
