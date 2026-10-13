use crate::{
    commons::article::Tag,
    features::article_parse::{article::Article, site::Site},
};
use std::path::Path;

use super::error::Error;

pub struct OgpGenerator;

impl OgpGenerator {
    pub fn generate_png(
        article: &Article,
        site: &Site,
        typst_project_root: impl AsRef<Path>,
        main_file: impl AsRef<Path>,
    ) -> Result<Vec<u8>, Error> {
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

        typst_html_export::compile_ogp_png(typst_project_root, main_file, &input).map_err(
            |source| Error::Typst {
                slug: article.slug.as_str().to_owned(),
                source,
            },
        )
    }
}
