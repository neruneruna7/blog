use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to load existing article inputs: {0}")]
    Input(#[from] crate::features::source_input::error::Error),
    #[error("article template loading failed: {0}")]
    LoadTemplate(#[from] crate::features::template_input::error::Error),
    #[error("article template artifact creation failed: {0}")]
    TemplateArtifact(#[from] crate::features::new_article_output::error::Error),
    #[error("article metadata is invalid: {0}")]
    Value(#[from] crate::commons::ValueError),
    #[error("failed to inspect article path '{path}': {source}")]
    InspectArticlePath {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not generate a unique article slug after {attempts} attempts under '{path}'")]
    UniqueSlugExhausted { path: PathBuf, attempts: usize },
}
