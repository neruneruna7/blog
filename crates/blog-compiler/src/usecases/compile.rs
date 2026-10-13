use std::collections::HashMap;

use crate::{
    commons::article::Slug,
    features::{
        article_parse::{article::Article, site::Site},
        article_transform::{models::mid::ProcessedArticle, transformer::ArticleTransformer},
        input::loader::Loader,
    },
};

pub mod error;

use error::Error;

pub(crate) fn article_compile() -> Result<HashMap<Slug, ProcessedArticle>, Error> {
    let input_bundle = Loader::load()?;
    let articles = Article::parse_all(input_bundle.articles)?;
    let site = Site::parse(input_bundle.site_config)?;

    Ok(ArticleTransformer::transform_all(articles, &site)?)
}
