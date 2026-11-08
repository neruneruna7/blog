use std::{io, path::PathBuf};

use thiserror::Error;

/// キャッシュManifestの更新中に発生するエラー。
#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to serialize cache manifest: {0}")]
    SerializeManifest(#[source] serde_json::Error),
    #[error("failed to write cache manifest '{path}': {source}")]
    WriteManifest {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cache fingerprint is not available for article '{0}'")]
    UnknownArticleSlug(String),
}
