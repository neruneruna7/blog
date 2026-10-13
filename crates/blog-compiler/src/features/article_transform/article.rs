use crate::{
    commons::article::Slug,
    features::article_parse::article::Article,
};

use super::{error::Error, output::ProcessedArticle, postprocess::Pipeline};

pub(super) fn transform(
    slug: Slug,
    article: Article,
    ogp_image: Vec<u8>,
    postprocess: &Pipeline,
) -> Result<(Slug, ProcessedArticle), Error> {
    let html = typst_html_export::compile_html(&article.root, &article.entry_point).map_err(
        |source| Error::TypstHtml {
            slug: article.slug.as_str().to_owned(),
            source,
        },
    )?;
    let postprocessed = postprocess.process(html, slug.as_str())?;

    Ok((
        slug,
        ProcessedArticle {
            article,
            html: postprocessed.body_html,
            ogp_image,
            warnings: postprocessed.warnings,
        },
    ))
}
