use std::collections::HashMap;

use crate::{
    commons::article::Slug,
    parse_article::{article::Article, site::Site},
};

use super::{
    error::Error,
    models::mid::{CompiledArticle, ProcessedArticle},
    ogp,
    postprocess::Pipeline,
    typst,
};

pub fn transform_all(
    articles: HashMap<Slug, Article>,
    site: &Site,
) -> Result<HashMap<Slug, ProcessedArticle>, Error> {
    let postprocess = Pipeline::production();
    articles
        .into_iter()
        .map(|(slug, article)| {
            let CompiledArticle { article, html } = typst::compile(article)?;
            let postprocessed = postprocess.process(html, slug.as_str())?;
            let ogp_image = ogp::generate_png(&article, site)?;

            Ok((
                slug,
                ProcessedArticle {
                    article,
                    html: postprocessed.body_html,
                    ogp_image,
                    warnings: postprocessed.warnings,
                },
            ))
        })
        .collect()
}
