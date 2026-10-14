use std::{collections::HashMap, path::Path};

use crate::{
    commons::article::{Slug, Tag},
    features::article_parse::{ParsedSiteBundle, article::Article, site::Site},
};

use super::error::Error;

pub struct OgpGenerator;

impl OgpGenerator {
    pub fn generate_all(
        parsed_site: &ParsedSiteBundle,
        typst_project_root: impl AsRef<Path>,
        main_file: impl AsRef<Path>,
    ) -> Result<HashMap<Slug, Vec<u8>>, Error> {
        let typst_project_root = typst_project_root.as_ref();
        let main_file = main_file.as_ref();

        parsed_site
            .articles
            .iter()
            .map(|(slug, article)| {
                let image =
                    Self::generate_png(article, &parsed_site.site, typst_project_root, main_file)?;
                Ok((slug.clone(), image))
            })
            .collect()
    }

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
