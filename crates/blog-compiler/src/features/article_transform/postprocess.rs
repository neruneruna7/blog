mod code_block;
mod extract_body_content;
mod web_link_card;

use super::{error::Error, models::mid::ArticleCompileWarning};

pub(super) struct HtmlPostprocessOutput {
    pub(super) body_html: String,
    pub(super) warnings: Vec<ArticleCompileWarning>,
}

/// htmlへの自前処理群
/// enumで多態性を表現
enum HtmlProcessor {
    CodeBlock(code_block::Processor),
    WebLinkCard(web_link_card::Processor),
    ExtractBodyContent(extract_body_content::Processor),
}

impl HtmlProcessor {
    /// lol_htmlはStringでhtml変換するので，Stringを受け取る
    fn process(
        &self,
        html: String,
        slug: &str,
    ) -> Result<(String, Vec<ArticleCompileWarning>), Error> {
        match self {
            Self::CodeBlock(processor) => Ok((processor.process(html, slug)?, Vec::new())),
            Self::WebLinkCard(processor) => {
                let output = processor.process(&html, slug)?;
                Ok((output.html, output.warnings))
            }
            Self::ExtractBodyContent(processor) => Ok((processor.process(html, slug)?, Vec::new())),
        }
    }
}

pub(super) struct Pipeline {
    processors: Vec<HtmlProcessor>,
}

impl Pipeline {
    pub(super) fn production() -> Self {
        Self {
            processors: vec![
                HtmlProcessor::CodeBlock(code_block::Processor),
                HtmlProcessor::WebLinkCard(web_link_card::Processor::production()),
                HtmlProcessor::ExtractBodyContent(extract_body_content::Processor),
            ],
        }
    }

    pub(super) fn process(&self, html: String, slug: &str) -> Result<HtmlPostprocessOutput, Error> {
        let (body_html, mut warnings) = self.processors.iter().try_fold(
            (html, Vec::new()),
            |(html, mut warnings), processor| {
                let (html, processor_warnings) = processor.process(html, slug)?;
                warnings.extend(processor_warnings);
                Ok::<_, Error>((html, warnings))
            },
        )?;
        warnings.sort_by(|left: &ArticleCompileWarning, right| left.url.cmp(&right.url));

        Ok(HtmlPostprocessOutput {
            body_html,
            warnings,
        })
    }
}
