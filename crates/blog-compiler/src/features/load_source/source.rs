use super::{article::RawArticle, site::RawSiteConfig};

#[derive(Debug, Clone)]
pub struct RawSource {
    pub articles: Vec<RawArticle>,
    pub site_config: RawSiteConfig,
}
