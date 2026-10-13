//! コードブロックを変換する
//! 自前のCSSを当てられるように

use crate::article_transform::error::Error;
use lol_html::{
    Settings, element, end_tag,
    html_content::{ContentType, TextChunk},
    rewrite_str, text,
};

const CODE_LINE_OPEN: &str = r#"<span class="code-line">"#;
const CODE_LINE_BREAK: &str = r#"</span><span class="code-line">"#;
const CODE_LINE_CLOSE: &str = "</span>";

pub(super) struct Processor;

impl Processor {
    pub(super) fn process(&self, html: String, slug: &str) -> Result<String, Error> {
        rewrite_str(
            &html,
            Settings {
                element_content_handlers: vec![
                    element!("pre > code", |element| {
                        element.prepend(CODE_LINE_OPEN, ContentType::Html);
                        element.on_end_tag(end_tag!(|end| {
                            end.before(CODE_LINE_CLOSE, ContentType::Html);
                            Ok(())
                        }))?;
                        Ok(())
                    }),
                    text!("pre > code", |text| {
                        replace_source_line_breaks(text);
                        Ok(())
                    }),
                ],
                ..Settings::default()
            },
        )
        .map_err(|error| Error::Postprocess {
            slug: slug.to_owned(),
            processor: "code block",
            reason: error.to_string(),
        })
    }
}

fn replace_source_line_breaks(text: &mut TextChunk<'_>) {
    let Some(replaced) = text
        .as_str()
        .contains('\n')
        .then(|| text.as_str().replace('\n', CODE_LINE_BREAK))
    else {
        return;
    };

    text.replace(&replaced, ContentType::Html);
}
