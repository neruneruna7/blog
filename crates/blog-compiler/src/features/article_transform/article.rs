use crate::{commons::article::Slug, features::source_parse::article::Article};

use super::{error::Error, postprocess::Pipeline};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformedArticle {
    pub article: Article,
    pub html: String,
    pub ogp_image: Vec<u8>,
    pub warnings: Vec<ArticleCompileWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleCompileWarning {
    pub url: String,
    pub reason: String,
}

pub(super) fn transform(
    slug: Slug,
    article: Article,
    ogp_image: Vec<u8>,
    postprocess: &Pipeline,
) -> Result<(Slug, TransformedArticle), Error> {
    tracing::debug!(slug = %slug.as_str(), "記事をHTMLへ変換");
    let html =
        typst_html_export::compile_html(&article.root, &article.entry_point).map_err(|source| {
            Error::TypstHtml {
                slug: article.slug.as_str().to_owned(),
                source,
            }
        })?;
    let postprocessed = postprocess.process(html, slug.as_str())?;
    for warning in &postprocessed.warnings {
        tracing::warn!(
            slug = %slug.as_str(),
            url = ?warning.url,
            reason = ?warning.reason,
            "記事の後処理で警告"
        );
    }

    Ok((
        slug,
        TransformedArticle {
            article,
            html: postprocessed.body_html,
            ogp_image,
            warnings: postprocessed.warnings,
        },
    ))
}
