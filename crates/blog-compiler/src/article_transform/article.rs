use crate::{
    commons::article::Slug,
    ogp_generate::OgpGenerator,
    parse_article::{article::Article, site::Site},
};

use super::{
    error::Error,
    models::mid::{CompiledArticle, ProcessedArticle},
    postprocess::Pipeline,
    typst,
};

pub(super) fn transform(
    slug: Slug,
    article: Article,
    site: &Site,
    postprocess: &Pipeline,
) -> Result<(Slug, ProcessedArticle), Error> {
    let CompiledArticle { article, html } = typst::compile(article)?;
    let postprocessed = postprocess.process(html, slug.as_str())?;
    let ogp_image = OgpGenerator::generate_png(&article, site)?;

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
