use std::collections::HashMap;

use crate::{
    article_transform::{models::mid::ProcessedArticle, transformer::ArticleTransformer},
    commons::article::Slug,
    input::loader::Loader,
    parse_article::{article::Article, site::Site},
};

pub mod error;

use error::Error;

pub(crate) fn article_compile() -> Result<HashMap<Slug, ProcessedArticle>, Error> {
    let input_bundle = Loader::load()?;
    let articles = Article::parse_all(input_bundle.articles)?;
    let site = Site::parse(input_bundle.site_config)?;

    Ok(ArticleTransformer::transform_all(articles, &site)?)
}
