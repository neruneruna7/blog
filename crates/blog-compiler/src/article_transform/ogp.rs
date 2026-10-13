use crate::{
    domain::article::Tag,
    domain_parse::{article::Article, site::Site},
    input::TYP_COMPILE_ROOT,
};

use super::error::Error;

pub const OGP_MAIN_FILE: &str = "commons/ogp/template.typ";

pub(super) fn generate_png(article: &Article, site: &Site) -> Result<Vec<u8>, Error> {
    let input = typst_html_export::OgpInput {
        site_title: site.title.as_str().to_owned(),
        article_title: article.title.as_str().to_owned(),
        description: article.description.as_str().to_owned(),
        tags: article
            .tags
            .iter()
            .map(Tag::as_str)
            .map(str::to_owned)
            .collect(),
    };

    typst_html_export::compile_ogp_png(TYP_COMPILE_ROOT, OGP_MAIN_FILE, &input).map_err(|source| {
        Error::Ogp {
            slug: article.slug.as_str().to_owned(),
            source,
        }
    })
}
