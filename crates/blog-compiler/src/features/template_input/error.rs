use std::{io, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read article template directory '{path}': {source}")]
    ReadTemplateDirectory {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to read an entry in article template directory '{path}': {source}")]
    ReadTemplateDirectoryEntry {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to inspect article template '{path}': {source}")]
    InspectTemplate {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("article template name is not valid UTF-8: '{path}'")]
    InvalidTemplateName { path: PathBuf },
    #[error("no Typst article templates were found under '{path}'")]
    NoTemplates { path: PathBuf },
    #[error("article template '{name}' was not found under '{path}'")]
    TemplateNotFound { name: String, path: PathBuf },
    #[error("failed to read article template '{path}': {source}")]
    ReadTemplate {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
