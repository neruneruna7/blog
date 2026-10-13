use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to compile article '{slug}' as HTML: {source}")]
    TypstHtml {
        slug: String,
        #[source]
        source: typst_html_export::TypstHtmlExportError,
    },
    #[error("failed to compile OGP image for article '{slug}': {source}")]
    Ogp {
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
}
