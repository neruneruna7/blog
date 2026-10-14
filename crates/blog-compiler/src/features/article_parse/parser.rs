use std::collections::HashMap;

use crate::{commons::article::Slug, features::load_article::site::RawSiteBundle};

use super::{article::Article, error::Error, site::Site};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSiteBundle {
    pub articles: HashMap<Slug, Article>,
    pub site: Site,
}

pub struct ArticleParser;

impl ArticleParser {
    pub fn parse(raw_site_bundle: RawSiteBundle) -> Result<ParsedSiteBundle, Error> {
        let articles = Article::parse_all(raw_site_bundle.articles)?;
        let site = Site::parse(raw_site_bundle.site_config)?;

        Ok(ParsedSiteBundle { articles, site })
    }
}
