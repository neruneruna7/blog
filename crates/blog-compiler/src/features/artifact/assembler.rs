use std::collections::HashMap;

use crate::{
    commons::article::Slug,
    features::{article_parse::site::Site, article_transform::TransformedArticle},
};

use super::{
    article::ArticleArtifact, artifact::Artifact, error::ArtifactBuildError, site::SiteConfig,
};

pub struct ArtifactAssembler;

impl ArtifactAssembler {
    pub fn assemble(
        transformed_articles: HashMap<Slug, TransformedArticle>,
        site: &Site,
    ) -> Result<Artifact, ArtifactBuildError> {
        let articles = transformed_articles
            .into_values()
            .map(|article| ArticleArtifact::try_from_transformed(article, site))
            .collect::<Result<Vec<_>, _>>()?;

        Artifact::try_new(SiteConfig::from(site), articles)
    }
}
