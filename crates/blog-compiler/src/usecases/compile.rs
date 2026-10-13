use std::collections::HashMap;

use crate::{
    commons::{
        app_config::{ARTICLE_ENTRY_POINT, OGP_MAIN_FILE, TYP_COMPILE_ROOT},
        article::Slug,
    },
    features::{
        article_parse::{article::Article, site::Site},
        article_transform::{models::mid::ProcessedArticle, transformer::ArticleTransformer},
        input::loader::Loader,
        ogp_generate::OgpGenerator,
    },
};

pub mod error;

use error::Error;

pub(crate) fn article_compile() -> Result<HashMap<Slug, ProcessedArticle>, Error> {
    let input_bundle = Loader::load(TYP_COMPILE_ROOT, ARTICLE_ENTRY_POINT)?;
    let articles = Article::parse_all(input_bundle.articles)?;
    let site = Site::parse(input_bundle.site_config)?;

    let ogp = articles
        .iter()
        .map(|(slug, article)| -> Result<_, Error> {
            Ok((
                slug.clone(),
                OgpGenerator::generate_png(article, &site, TYP_COMPILE_ROOT, OGP_MAIN_FILE)?,
            ))
        })
        .collect::<Result<HashMap<_, _>, Error>>()?;
    let transformed =
        ArticleTransformer::transform_all(articles, &site, TYP_COMPILE_ROOT, OGP_MAIN_FILE)?;

    todo!()
}
