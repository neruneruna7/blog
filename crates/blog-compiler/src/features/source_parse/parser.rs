use crate::features::load_source::source::RawSource;

use super::{article::Article, error::Error, site::Site, source::ParsedSource};

pub struct SourceParser;

impl SourceParser {
    pub fn parse(raw_source: RawSource) -> Result<ParsedSource, Error> {
        let articles = Article::parse_all(raw_source.articles)?;
        let site = Site::parse(raw_source.site_config)?;

        Ok(ParsedSource { articles, site })
    }
}
