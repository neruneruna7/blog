use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to write article HTML '{path}': {source}")]
    WriteHtml {
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
    #[error("failed to remove obsolete article HTML '{path}': {source}")]
    RemoveObsoleteHtml {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
