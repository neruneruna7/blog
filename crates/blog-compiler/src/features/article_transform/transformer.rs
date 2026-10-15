use std::collections::HashMap;

use crate::{
    commons::article::Slug,
    features::source_parse::article::Article,
};

use super::{article::TransformedArticle, error::Error, postprocess::Pipeline};

pub struct ArticleTransformer;

impl ArticleTransformer {
    pub fn transform_all(
        articles: HashMap<Slug, Article>,
        mut ogp_images: HashMap<Slug, Vec<u8>>,
    ) -> Result<HashMap<Slug, TransformedArticle>, Error> {
        let postprocess = Pipeline::production();
        let transformed = articles
            .into_iter()
            .map(|(slug, article)| {
                if slug != article.slug {
                    return Err(Error::ArticleSlugMismatch {
                        key_slug: slug.as_str().to_owned(),
                        article_slug: article.slug.as_str().to_owned(),
                    });
                }

                let ogp_image = ogp_images
                    .remove(&slug)
                    .ok_or_else(|| Error::MissingOgpImage(slug.as_str().to_owned()))?;
                Self::transform_article(article, ogp_image, &postprocess)
                    .map(|transformed_article| (slug, transformed_article))
            })
            .collect::<Result<HashMap<_, _>, Error>>()?;

        if let Some(slug) = ogp_images.into_keys().next() {
            return Err(Error::OgpImageWithoutArticle(slug.as_str().to_owned()));
        }

        Ok(transformed)
    }

    fn transform_article(
        article: Article,
        ogp_image: Vec<u8>,
        postprocess: &Pipeline,
    ) -> Result<TransformedArticle, Error> {
        let Article {
            root,
            entry_point,
            slug,
            title,
            description,
            tags,
            published_at,
            updated_at,
        } = article;

        tracing::debug!(slug = %slug.as_str(), "記事をHTMLへ変換");
        let html = typst_html_export::compile_html(&root, &entry_point).map_err(|source| {
            Error::TypstHtml {
                slug: slug.as_str().to_owned(),
                source,
            }
        })?;
        let postprocessed = postprocess.process(html, slug.as_str())?;
        for warning in &postprocessed.warnings {
            tracing::warn!(
                slug = %slug.as_str(),
                url = ?warning.url,
                reason = ?warning.reason,
                "記事の後処理で警告"
            );
        }

        Ok(TransformedArticle {
            slug,
            title,
            description,
            tags,
            published_at,
            updated_at,
            html: postprocessed.body_html,
            ogp_image,
            warnings: postprocessed.warnings,
        })
    }
}
