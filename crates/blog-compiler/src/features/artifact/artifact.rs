use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use serde::Serialize;

use super::{
    article::ArticleArtifact,
    error::{ArtifactBuildError, ArtifactWriteError},
    site::SiteConfig,
};

const ARTICLES_DIRECTORY: &str = "articles";
const PUBLIC_ROOT_DIRECTORY: &str = "public-root";
const BODY_HTML_FILE: &str = "body.html";
const ARTICLE_METADATA_FILE: &str = "meta.json";
const SITE_CONFIG_FILE: &str = "site.json";
const ARTICLE_INDEX_FILE: &str = "article-index.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Artifact {
    pub config: SiteConfig,
    pub articles: Vec<ArticleArtifact>,
}

impl Artifact {
    pub fn try_new(
        config: SiteConfig,
        articles: Vec<ArticleArtifact>,
    ) -> Result<Self, ArtifactBuildError> {
        let mut slugs = HashSet::new();
        for article in &articles {
            if !slugs.insert(article.slug.clone()) {
                return Err(ArtifactBuildError::DuplicateArticleSlug(
                    article.slug.as_str().to_owned(),
                ));
            }
        }

        Ok(Self { config, articles })
    }

    pub fn write(&self, output_root: impl AsRef<Path>) -> Result<(), ArtifactWriteError> {
        let output_root = output_root.as_ref();
        create_directory(output_root)?;
        write_json(&output_root.join(SITE_CONFIG_FILE), &self.config)?;

        let article_index = self
            .articles
            .iter()
            .map(|article| &article.metadata)
            .collect::<Vec<_>>();
        write_json(&output_root.join(ARTICLE_INDEX_FILE), &article_index)?;

        for article in &self.articles {
            write_article(article, output_root)?;
        }

        Ok(())
    }
}

fn write_article(article: &ArticleArtifact, output_root: &Path) -> Result<(), ArtifactWriteError> {
    let article_directory = output_root
        .join(ARTICLES_DIRECTORY)
        .join(article.slug.as_str());
    create_directory(&article_directory)?;

    let body_html = format!("{}\n", article.html);
    write_file(
        &article_directory.join(BODY_HTML_FILE),
        body_html.as_bytes(),
    )?;
    write_json(
        &article_directory.join(ARTICLE_METADATA_FILE),
        &article.metadata,
    )?;

    for asset in &article.assets {
        let asset_path = output_root
            .join(PUBLIC_ROOT_DIRECTORY)
            .join(&asset.relative_path);
        if let Some(parent) = asset_path.parent() {
            create_directory(parent)?;
        }
        write_file(&asset_path, &asset.bytes)?;
    }

    Ok(())
}

fn create_directory(path: &Path) -> Result<(), ArtifactWriteError> {
    fs::create_dir_all(path).map_err(|source| ArtifactWriteError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, contents: &[u8]) -> Result<(), ArtifactWriteError> {
    fs::write(path, contents).map_err(|source| ArtifactWriteError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn write_json(path: &Path, value: &(impl Serialize + ?Sized)) -> Result<(), ArtifactWriteError> {
    let mut contents =
        serde_json::to_vec_pretty(value).map_err(|source| ArtifactWriteError::SerializeJson {
            path: path.to_path_buf(),
            source,
        })?;
    contents.push(b'\n');
    write_file(path, &contents)
}
