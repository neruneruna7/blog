use crate::features::article_parse::article::Article;

use super::error::Error;

use super::models::mid::CompiledArticle;

pub(super) fn compile(article: Article) -> Result<CompiledArticle, Error> {
    let html =
        typst_html_export::compile_html(&article.root, &article.entry_point).map_err(|source| {
            Error::TypstHtml {
                slug: article.slug.as_str().to_owned(),
                source,
            }
        })?;

    Ok(CompiledArticle { article, html })
}
