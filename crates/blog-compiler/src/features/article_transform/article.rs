use crate::{
    commons::article::Slug,
    features::{
        article_parse::{article::Article, site::Site},
        ogp_generate::OgpGenerator,
    },
};
use std::path::Path;

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
    ogp_project_root: &Path,
    ogp_main_file: &Path,
    postprocess: &Pipeline,
) -> Result<(Slug, ProcessedArticle), Error> {
    let CompiledArticle { article, html } = typst::compile(article)?;
    let postprocessed = postprocess.process(html, slug.as_str())?;
    let ogp_image = OgpGenerator::generate_png(&article, site, ogp_project_root, ogp_main_file)?;

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
