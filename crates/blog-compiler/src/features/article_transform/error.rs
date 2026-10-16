use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to compile article '{slug}' as HTML: {source}")]
    TypstHtml {
        slug: String,
        #[source]
        source: typst_html_export::TypstHtmlExportError,
    },
    #[error("failed to postprocess article '{slug}' in {processor}: {reason}")]
    Postprocess {
        slug: String,
        processor: &'static str,
        reason: String,
    },
    #[error("OGP image is missing for article '{0}'")]
    MissingOgpImage(String),
    #[error("OGP image was provided for unknown article '{0}'")]
    OgpImageWithoutArticle(String),
    #[error("article map key '{key_slug}' does not match its slug '{article_slug}'")]
    ArticleSlugMismatch {
        key_slug: String,
        article_slug: String,
    },
}
