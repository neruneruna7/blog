use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to inspect cache input directory '{path}': {source}")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect cache input '{path}': {source}")]
    InspectInput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read cache input '{path}': {source}")]
    ReadInput {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("symbolic links are not supported in cache input trees: '{0}'")]
    SymbolicLink(PathBuf),
    #[error("failed to serialize OGP cache input: {0}")]
    SerializeHashInput(#[from] serde_json::Error),
}
