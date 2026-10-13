use std::collections::{HashMap, HashSet};

use crate::{
    error::{ArticleCompileError, ArticleValidationError},
    input::{self, article::RawArticle, site::RawSiteBundle},
};
use chrono::{DateTime, FixedOffset};

pub mod models;

use models::mid::{
    ArticleId, CompiledArticle, Description, PublishedAt, Slug, Tag, Title, ValidatedArticle,
};

pub(crate) fn article_compile() -> Result<HashMap<Slug, CompiledArticle>, ArticleCompileError> {
    // let raw_articles = RawArticle::read_all(input::TYP_COMPILE_ROOT)?;
    let input_bundle = RawSiteBundle::read()?;
    let validated_articles = RawArticle::validate_all(raw_articles)?;
    let compiled_articles = validated_articles
        .into_iter()
        .map(|article| {
            let slug = article.slug.clone();
            article.compile().map(|compiled| (slug, compiled))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;

    Ok(compiled_articles)
}

const MAX_TITLE_CHARACTERS: usize = 48;
const MAX_DESCRIPTION_CHARACTERS: usize = 130;

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
            return Err(ArticleValidationError::DuplicateSlug(slug.0));
        }

        Ok(ValidatedArticle {
            root: self.root,
            entry_point: self.entry_point,
            id: ArticleId::new(self.config.id)?,
            slug,
            title: Title::new(self.config.title)?,
            description: Description::new(self.config.description)?,
            tags: self.config.tags.into_iter().map(Tag).collect(),
            published_at: PublishedAt::new(self.config.published_at)?,
            updated_at: parse_optional_datetime("updated_at", self.config.updated_at)?,
        })
    }
}

impl ValidatedArticle {
    fn compile(self) -> Result<CompiledArticle, ArticleCompileError> {
        let html =
            typst_html_export::compile_html(&self.root, &self.entry_point).map_err(|source| {
                ArticleCompileError::TypstHtml {
                    slug: self.slug.0.clone(),
                    source,
                }
            })?;

        Ok(CompiledArticle {
            article: self,
            html,
        })
    }
}

impl Slug {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        let is_valid = value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        });
        if !is_valid {
            return Err(ArticleValidationError::InvalidSlug(value));
        }

        Ok(Self(value))
    }
}

impl ArticleId {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        validate_non_empty("id", &value)?;
        Ok(Self(value))
    }
}

impl Title {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        validate_text_length("title", &value, MAX_TITLE_CHARACTERS)?;
        Ok(Self(value))
    }
}

impl Description {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        validate_text_length("description", &value, MAX_DESCRIPTION_CHARACTERS)?;
        Ok(Self(value))
    }
}

impl PublishedAt {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        DateTime::<FixedOffset>::parse_from_rfc3339(&value)
            .map(Self)
            .map_err(|source| ArticleValidationError::InvalidDateTime {
                field: "published_at",
                source,
            })
    }
}

fn parse_optional_datetime(
    field: &'static str,
    value: String,
) -> Result<Option<DateTime<FixedOffset>>, ArticleValidationError> {
    if value.is_empty() {
        return Ok(None);
    }

    DateTime::<FixedOffset>::parse_from_rfc3339(&value)
        .map(Some)
        .map_err(|source| ArticleValidationError::InvalidDateTime { field, source })
}

fn validate_text_length(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), ArticleValidationError> {
    validate_non_empty(field, value)?;
    let actual = value.chars().count();
    if actual > maximum {
        return Err(ArticleValidationError::TextTooLong {
            field,
            maximum,
            actual,
        });
    }

    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ArticleValidationError> {
    if value.is_empty() {
        return Err(ArticleValidationError::EmptyValue { field });
    }

    Ok(())
}
