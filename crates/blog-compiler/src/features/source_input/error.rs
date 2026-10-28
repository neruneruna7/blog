use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read article directory '{path}': {source}")]
    ReadDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read an entry in article directory '{path}': {source}")]
    ReadDirectoryEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect article directory entry '{path}': {source}")]
    InspectDirectoryEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read article configuration '{path}': {source}")]
    ReadArticleConfig {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse article configuration '{path}': {source}")]
    ParseArticleConfig {
        path: PathBuf,
        #[source]
        source: super::toml_config::ConfigError,
    },
    #[error("failed to read site configuration '{path}': {source}")]
    ReadSiteConfig {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to parse site configuration '{path}': {source}")]
    ParseSiteConfig {
        path: PathBuf,
        #[source]
        source: super::toml_config::ConfigError,
    },
}
