//! ページ文書と記事本文を分けるため、`<body>` の内側を抽出する。

use regex::Regex;

use crate::features::article_transform::error::Error;

pub(super) struct Processor;

impl Processor {
    pub(super) fn process(&self, html: String, slug: &str) -> Result<String, Error> {
        extract(&html).ok_or_else(|| Error::Postprocess {
            slug: slug.to_owned(),
            processor: "body content extraction",
            reason: "HTML document does not contain a complete <body> element".to_owned(),
        })
    }
}

fn extract(html: &str) -> Option<String> {
    let body_content =
        Regex::new(r"(?is)<body\b[^>]*>(.*?)</body\s*>").expect("body content regex is valid");

    body_content
        .captures(html)
        .and_then(|captures| captures.get(1))
        .map(|body| body.as_str().trim().to_owned())
}
