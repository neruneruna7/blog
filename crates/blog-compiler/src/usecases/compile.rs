use std::collections::HashMap;

use crate::{
    article_transform::{self, models::mid::ProcessedArticle},
    domain::article::Slug,
    domain_parse::{article::Article, site::Site},
    input::loader::Loader,
};

pub mod error;

use error::Error;

pub(crate) fn article_compile() -> Result<HashMap<Slug, ProcessedArticle>, Error> {
    let input_bundle = Loader::load()?;
    let articles = Article::parse_all(input_bundle.articles)?;
    let site = Site::parse(input_bundle.site_config)?;

    Ok(article_transform::article::transform_all(articles, &site)?)
}
