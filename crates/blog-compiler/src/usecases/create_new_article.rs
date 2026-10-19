use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use uuid::Uuid;

use crate::{
    commons::{
        app_config::{ARTICLE_ENTRY_POINT, TYP_COMPILE_ROOT},
        article::{ArticleTitle, Description, Slug, Tag},
    },
    features::{
        load_source::loader::SourceLoader,
        load_template::loader::TemplateLoader,
        template_artifact::{article::NewArticle, config::ArticleConfig},
    },
};

pub mod error;

use error::Error;

const ARTICLE_FOLDER: &str = "articles";
const TEMPLATE_FOLDER: &str = "commons/template";
const SLUG_LENGTH: usize = 8;
const MAX_SLUG_ATTEMPTS: usize = 16;

#[derive(Debug, Clone, Default)]
pub struct NewArticleRequest {
    pub template: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedArticle {
    pub slug: String,
    pub article_dir: PathBuf,
    pub template: String,
}

#[tracing::instrument(skip(request), err)]
pub fn create_new_article(request: NewArticleRequest) -> Result<CreatedArticle, Error> {
    let source_root = Path::new(TYP_COMPILE_ROOT);
    let template_root = source_root.join(TEMPLATE_FOLDER);
    let template = TemplateLoader::load_selected(&template_root, request.template.as_deref())?;
    let template_name = template.name().to_owned();

    let source = SourceLoader::load(source_root, ARTICLE_ENTRY_POINT)?;
    let existing_slugs = source
        .articles
        .into_iter()
        .map(|article| article.config.slug)
        .collect::<HashSet<_>>();

    let articles_root = source_root.join(ARTICLE_FOLDER);
    let slug = select_unique_slug(&articles_root, &existing_slugs)?;
    let config = build_article_config(request, slug)?;
    let slug = config.slug.clone();
    let article_dir = articles_root.join(&slug);

    let new_article = NewArticle::new(
        source_root.to_path_buf(),
        article_dir.clone(),
        template,
        config,
    );
    new_article.create()?;

    Ok(CreatedArticle {
        slug,
        article_dir,
        template: template_name,
    })
}

fn select_unique_slug(
    articles_root: &Path,
    existing_slugs: &HashSet<String>,
) -> Result<Slug, Error> {
    for _ in 0..MAX_SLUG_ATTEMPTS {
        let raw_slug = Uuid::now_v7().simple().to_string();
        let slug = Slug::new(raw_slug[raw_slug.len() - SLUG_LENGTH..].to_owned())?;
        let article_path = articles_root.join(slug.as_str());
        let path_exists =
            article_path
                .try_exists()
                .map_err(|source| Error::InspectArticlePath {
                    path: article_path,
                    source,
                })?;

        if !existing_slugs.contains(slug.as_str()) && !path_exists {
            return Ok(slug);
        }
    }

    Err(Error::UniqueSlugExhausted {
        path: articles_root.to_path_buf(),
        attempts: MAX_SLUG_ATTEMPTS,
    })
}

fn build_article_config(request: NewArticleRequest, slug: Slug) -> Result<ArticleConfig, Error> {
    let mut config = ArticleConfig::with_slug(slug.into_string());

    let title_value = request
        .title
        .unwrap_or_else(|| config.title.clone())
        .trim()
        .to_owned();
    config.title = ArticleTitle::new(title_value)?.into_string();

    let description_value = request
        .description
        .unwrap_or_else(|| config.title.clone())
        .trim()
        .to_owned();
    config.description = Description::new(description_value)?.into_string();
    config.tags = request
        .tags
        .into_iter()
        .map(|tag| Tag::new(tag.trim().to_owned()).into_string())
        .collect();

    Ok(config)
}
