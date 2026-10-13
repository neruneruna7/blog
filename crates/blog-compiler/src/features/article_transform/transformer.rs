use std::collections::HashMap;
use std::path::Path;

use crate::{
    commons::article::Slug,
    features::article_parse::{article::Article, site::Site},
};

use super::{article, error::Error, models::mid::ProcessedArticle, postprocess::Pipeline};

pub struct ArticleTransformer;

impl ArticleTransformer {
    pub fn transform_all(
        articles: HashMap<Slug, Article>,
        site: &Site,
        ogp_project_root: impl AsRef<Path>,
        ogp_main_file: impl AsRef<Path>,
    ) -> Result<HashMap<Slug, ProcessedArticle>, Error> {
        let postprocess = Pipeline::production();
        let ogp_project_root = ogp_project_root.as_ref();
        let ogp_main_file = ogp_main_file.as_ref();
        articles
            .into_iter()
            .map(|(slug, article)| {
                article::transform(
                    slug,
                    article,
                    site,
                    ogp_project_root,
                    ogp_main_file,
                    &postprocess,
                )
            })
            .collect()
    }
}
