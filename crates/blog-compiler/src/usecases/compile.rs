use std::collections::{HashMap, HashSet};

use crate::{
    domain::article::{ArticleId, ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt},
    error::{ArticleCompileError, ArticleValidationError},
    input::{article::RawArticle, site::RawSiteBundle},
};

pub mod models;

use models::mid::{CompiledArticle, ValidatedArticle};

pub(crate) fn article_compile() -> Result<HashMap<Slug, CompiledArticle>, ArticleCompileError> {
    let input_bundle = RawSiteBundle::read()?;
    let validated_articles = RawArticle::validate_all(input_bundle.articles)?;
    let compiled_articles = validated_articles
        .into_iter()
        .map(|article| {
            let slug = article.slug.clone();
            article.compile().map(|compiled| (slug, compiled))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(compiled_articles)
}

impl RawArticle {
    fn validate_all(
        articles: impl IntoIterator<Item = Self>,
    ) -> Result<Vec<ValidatedArticle>, ArticleValidationError> {
        let mut seen_slugs = HashSet::new();
        articles
            .into_iter()
            .map(|article| article.validate(&mut seen_slugs))
            .collect()
    }

    fn validate(
        self,
        seen_slugs: &mut HashSet<Slug>,
    ) -> Result<ValidatedArticle, ArticleValidationError> {
        let slug = Slug::new(self.config.slug)?;
        if !seen_slugs.insert(slug.clone()) {
            return Err(ArticleValidationError::DuplicateSlug(slug.into_string()));
        }

        Ok(ValidatedArticle {
            root: self.root,
            entry_point: self.entry_point,
            id: ArticleId::new(self.config.id)?,
            slug,
            title: ArticleTitle::new(self.config.title)?,
            description: Description::new(self.config.description)?,
            tags: self.config.tags.into_iter().map(Tag::new).collect(),
            published_at: PublishedAt::new(self.config.published_at)?,
            updated_at: UpdatedAt::new(self.config.updated_at)?,
        })
    }
}

impl ValidatedArticle {
    fn compile(self) -> Result<CompiledArticle, ArticleCompileError> {
        let html =
            typst_html_export::compile_html(&self.root, &self.entry_point).map_err(|source| {
                ArticleCompileError::TypstHtml {
                    slug: self.slug.as_str().to_owned(),
                    source,
                }
            })?;

        Ok(CompiledArticle {
            article: self,
            html,
        })
    }
}
