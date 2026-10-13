use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to compile article '{slug}' as HTML: {source}")]
    TypstHtml {
        slug: String,
        #[source]
        source: typst_html_export::TypstHtmlExportError,
    },
    #[error("OGP image generation failed: {0}")]
    OgpGeneration(#[from] crate::ogp_generate::error::Error),
    #[error("failed to postprocess article '{slug}' in {processor}: {reason}")]
    Postprocess {
        slug: String,
        processor: &'static str,
        reason: String,
    },
}
