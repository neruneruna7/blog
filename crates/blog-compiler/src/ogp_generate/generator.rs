use crate::{
    commons::article::Tag,
    input::TYP_COMPILE_ROOT,
    parse_article::{article::Article, site::Site},
};

use super::error::Error;

const OGP_MAIN_FILE: &str = "commons/ogp/template.typ";

pub struct OgpGenerator;

impl OgpGenerator {
    pub fn generate_png(article: &Article, site: &Site) -> Result<Vec<u8>, Error> {
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

        typst_html_export::compile_ogp_png(TYP_COMPILE_ROOT, OGP_MAIN_FILE, &input).map_err(
            |source| Error::Typst {
                slug: article.slug.as_str().to_owned(),
                source,
            },
        )
    }
}
