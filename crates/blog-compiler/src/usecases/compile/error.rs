use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read article inputs: {0}")]
    Input(#[from] crate::features::load_article::error::Error),
    #[error("domain parsing failed: {0}")]
    DomainParse(#[from] crate::features::article_parse::error::Error),
    #[error("article transformation failed: {0}")]
    ArticleTransform(#[from] crate::features::article_transform::error::Error),
    #[error("OGP generation failed: {0}")]
    OgpGenerate(#[from] crate::features::ogp_generate::error::Error),
    #[error("artifact construction failed: {0}")]
    ArtifactBuild(#[from] crate::features::artifact::error::ArtifactBuildError),
    #[error("artifact writing failed: {0}")]
    ArtifactWrite(#[from] crate::features::artifact::error::ArtifactWriteError),
}
