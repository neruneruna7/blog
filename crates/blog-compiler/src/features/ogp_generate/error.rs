use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to generate OGP image for article '{slug}': {source}")]
    Typst {
        slug: String,
        #[source]
        source: typst_html_export::TypstHtmlExportError,
    },
}
