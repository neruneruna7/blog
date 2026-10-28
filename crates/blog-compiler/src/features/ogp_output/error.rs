use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to write OGP image '{path}': {source}")]
    WriteImage {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect generated OGP directory '{path}': {source}")]
    ReadOgpDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect generated OGP image '{path}': {source}")]
    InspectOgpImage {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to remove obsolete OGP image '{path}': {source}")]
    RemoveObsoleteImage {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
