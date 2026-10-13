use std::{collections::HashSet, path::Path};

use serde::Serialize;

use crate::commons::article::{
    ArticleId, ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt,
};

use super::{
    create_directory,
    error::{ArtifactBuildError, ArtifactWriteError},
    write_file, write_json,
};

const ARTICLES_DIRECTORY: &str = "articles";
const PUBLIC_ROOT_DIRECTORY: &str = "public-root";
const BODY_HTML_FILE: &str = "body.html";
const ARTICLE_METADATA_FILE: &str = "meta.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleArtifact {
    pub slug: Slug,
    pub html: String,
    pub metadata: ArticleMetadata,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleMetadata {
    pub id: ArticleId,
    pub slug: Slug,
    pub title: ArticleTitle,
    pub description: Description,
    pub published_at: PublishedAt,
    pub updated_at: UpdatedAt,
    pub tags: Vec<Tag>,
    pub url: String,
    pub ogp_image: OgpImageMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OgpImageMetadata {
    pub path: String,
    pub width: u32,
    pub height: u32,
    pub mime_type: String,
    pub alt: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    relative_path: std::path::PathBuf,
    bytes: Vec<u8>,
}

impl Asset {
    pub fn try_new(
        relative_path: impl Into<std::path::PathBuf>,
        bytes: Vec<u8>,
    ) -> Result<Self, ArtifactBuildError> {
        let relative_path = relative_path.into();
        ensure_relative_asset_path(&relative_path)?;

        Ok(Self {
            relative_path,
            bytes,
        })
    }

    pub fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl ArticleArtifact {
    pub fn try_new(
        slug: Slug,
        html: String,
        metadata: ArticleMetadata,
        assets: Vec<Asset>,
    ) -> Result<Self, ArtifactBuildError> {
        if slug != metadata.slug {
            return Err(ArtifactBuildError::ArticleSlugMismatch {
                artifact_slug: slug.as_str().to_owned(),
                metadata_slug: metadata.slug.as_str().to_owned(),
            });
        }

        let mut asset_paths = HashSet::new();
        for asset in &assets {
            if !asset_paths.insert(asset.relative_path.clone()) {
                return Err(ArtifactBuildError::DuplicateAssetPath {
                    slug: slug.as_str().to_owned(),
                    path: asset.relative_path.clone(),
                });
            }
        }

        let ogp_path = parse_ogp_asset_path(&metadata.ogp_image.path)?;
        if !asset_paths.contains(&ogp_path) {
            return Err(ArtifactBuildError::MissingOgpAsset {
                slug: slug.as_str().to_owned(),
                path: ogp_path,
            });
        }

        Ok(Self {
            slug,
            html,
            metadata,
            assets,
        })
    }

    pub fn write(&self, output_root: impl AsRef<Path>) -> Result<(), ArtifactWriteError> {
        let output_root = output_root.as_ref();
        let article_directory = output_root
            .join(ARTICLES_DIRECTORY)
            .join(self.slug.as_str());
        create_directory(&article_directory)?;

        let body_html_path = article_directory.join(BODY_HTML_FILE);
        let body_html = format!("{}\n", self.html);
        write_file(&body_html_path, body_html.as_bytes())?;

        write_json(
            &article_directory.join(ARTICLE_METADATA_FILE),
            &self.metadata,
        )?;

        for asset in &self.assets {
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
}

fn parse_ogp_asset_path(value: &str) -> Result<std::path::PathBuf, ArtifactBuildError> {
    let path = value
        .strip_prefix('/')
        .map(Path::new)
        .ok_or_else(|| ArtifactBuildError::InvalidAssetPath(value.into()))?;
    ensure_relative_asset_path(path)?;

    Ok(path.to_path_buf())
}

fn ensure_relative_asset_path(path: &Path) -> Result<(), ArtifactBuildError> {
    let mut components = path.components().peekable();
    if components.peek().is_none()
        || components.any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(ArtifactBuildError::InvalidAssetPath(path.to_path_buf()));
    }

    Ok(())
}
