use std::collections::HashMap;

use crate::{
    commons::article::Slug,
    parse_article::{article::Article, site::Site},
};

use super::{article, error::Error, models::mid::ProcessedArticle, postprocess::Pipeline};

pub struct ArticleTransformer;

impl ArticleTransformer {
    pub fn transform_all(
        articles: HashMap<Slug, Article>,
        site: &Site,
    ) -> Result<HashMap<Slug, ProcessedArticle>, Error> {
        let postprocess = Pipeline::production();
        articles
            .into_iter()
            .map(|(slug, article)| article::transform(slug, article, site, &postprocess))
            .collect()
    }
}
