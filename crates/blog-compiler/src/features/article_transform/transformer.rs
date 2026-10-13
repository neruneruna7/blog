use std::collections::HashMap;

use crate::{
    commons::article::Slug,
    features::article_parse::article::Article,
};

use super::{article, error::Error, output::ProcessedArticle, postprocess::Pipeline};

pub struct ArticleTransformer;

impl ArticleTransformer {
    pub fn transform_all(
        articles: HashMap<Slug, Article>,
        mut ogp_images: HashMap<Slug, Vec<u8>>,
    ) -> Result<HashMap<Slug, ProcessedArticle>, Error> {
        let postprocess = Pipeline::production();
        let transformed = articles
            .into_iter()
            .map(|(slug, article)| {
                let ogp_image = ogp_images
                    .remove(&slug)
                    .ok_or_else(|| Error::MissingOgpImage(slug.as_str().to_owned()))?;
                article::transform(
                    slug,
                    article,
                    ogp_image,
                    &postprocess,
                )
            })
            .collect::<Result<HashMap<_, _>, Error>>()?;

        if let Some(slug) = ogp_images.into_keys().next() {
            return Err(Error::OgpImageWithoutArticle(slug.as_str().to_owned()));
        }

        Ok(transformed)
    }
}
