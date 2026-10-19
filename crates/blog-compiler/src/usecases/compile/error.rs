use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to read article inputs: {0}")]
    Input(#[from] crate::features::load_source::error::Error),
    #[error("build cache manifest writing failed: {0}")]
    BuildCacheWriter(#[from] crate::features::build_cache_writer::Error),
    #[error("blog cache input loading failed: {0}")]
    BlogCacheLoad(#[from] crate::features::blog_cache_loader::error::Error),
    #[error("domain parsing failed: {0}")]
    DomainParse(#[from] crate::features::source_parse::error::Error),
    #[error("article transformation failed: {0}")]
    ArticleTransform(#[from] crate::features::article_transform::error::Error),
    #[error("OGP generation failed: {0}")]
    OgpGenerate(#[from] crate::features::ogp_generate::error::Error),
    #[error("article HTML writing failed: {0}")]
    ArticleWrite(#[from] crate::features::article_write::error::Error),
    #[error("OGP image writing failed: {0}")]
    OgpWrite(#[from] crate::features::ogp_write::error::Error),
    #[error("metadata writing failed: {0}")]
    MetadataWrite(#[from] crate::features::metadata_write::error::Error),
}
