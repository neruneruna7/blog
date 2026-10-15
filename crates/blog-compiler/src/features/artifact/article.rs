use std::{collections::HashSet, path::Path};

use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::commons::article::{ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt};
use crate::features::{
    article_transform::TransformedArticle,
    source_parse::{article::Article, site::Site},
};

use super::error::ArtifactBuildError;

const OGP_ASSET_DIRECTORY: &str = "ogp/articles";
const OGP_IMAGE_MIME_TYPE: &str = "image/png";

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
    pub(super) relative_path: std::path::PathBuf,
    pub(super) bytes: Vec<u8>,
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
}

impl ArticleArtifact {
    pub fn try_from_transformed(
        transformed: TransformedArticle,
        site: &Site,
    ) -> Result<Self, ArtifactBuildError> {
        let TransformedArticle {
            article,
            html,
            ogp_image,
            ..
        } = transformed;
        let Article {
            slug,
            title,
            description,
            tags,
            published_at,
            updated_at,
            ..
        } = article;

        let digest = Sha256::digest(&ogp_image);
        let content_hash = format!("{digest:x}");
        let file_name = format!("{}-{}.png", slug.as_str(), &content_hash[..12]);
        let relative_asset_path = Path::new(OGP_ASSET_DIRECTORY).join(&file_name);
        let public_image_path = format!("/{OGP_ASSET_DIRECTORY}/{file_name}");
        let alt = format!("{} | {}", title.as_str(), site.title.as_str());
        let asset = Asset::try_new(relative_asset_path, ogp_image)?;
        let metadata = ArticleMetadata {
            slug: slug.clone(),
            title,
            description,
            published_at,
            updated_at,
            tags,
            url: format!("/articles/{}/", slug.as_str()),
            ogp_image: OgpImageMetadata {
                path: public_image_path,
                width: typst_html_export::OGP_WIDTH_PX,
                height: typst_html_export::OGP_HEIGHT_PX,
                mime_type: OGP_IMAGE_MIME_TYPE.to_owned(),
                alt,
            },
        };

        Self::try_new(slug, html, metadata, vec![asset])
    }

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
