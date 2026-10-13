use std::{collections::HashSet, path::Path};

use serde::Serialize;

use super::{create_directory, error::Error, write_file, write_json};

const ARTICLES_DIRECTORY: &str = "articles";
const PUBLIC_ROOT_DIRECTORY: &str = "public-root";
const BODY_HTML_FILE: &str = "body.html";
const ARTICLE_METADATA_FILE: &str = "meta.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArticleBundle {
    pub slug: String,
    pub html: String,
    pub metadata: ArticleMetadata,
    pub assets: Vec<Asset>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleMetadata {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub updated_at: Option<String>,
    pub tags: Vec<String>,
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
    pub relative_path: std::path::PathBuf,
    pub bytes: Vec<u8>,
}

impl ArticleBundle {
    pub fn write(&self, output_root: impl AsRef<Path>) -> Result<(), Error> {
        self.validate()?;

        let output_root = output_root.as_ref();
        let article_directory = output_root.join(ARTICLES_DIRECTORY).join(&self.slug);
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

    pub(super) fn validate(&self) -> Result<(), Error> {
        validate_path_segment(&self.slug)?;
        if self.slug != self.metadata.slug {
            return Err(Error::ArticleSlugMismatch {
                bundle_slug: self.slug.clone(),
                metadata_slug: self.metadata.slug.clone(),
            });
        }

        let mut asset_paths = HashSet::new();
        for asset in &self.assets {
            validate_relative_asset_path(&asset.relative_path)?;
            if !asset_paths.insert(asset.relative_path.as_path()) {
                return Err(Error::DuplicateAssetPath {
                    slug: self.slug.clone(),
                    path: asset.relative_path.clone(),
                });
            }
        }

        let ogp_path = self
            .metadata
            .ogp_image
            .path
            .strip_prefix('/')
            .map(Path::new)
            .ok_or_else(|| Error::InvalidAssetPath(self.metadata.ogp_image.path.clone().into()))?;
        validate_relative_asset_path(ogp_path)?;
        if !asset_paths.contains(ogp_path) {
            return Err(Error::MissingOgpAsset {
                slug: self.slug.clone(),
                path: ogp_path.to_path_buf(),
            });
        }

        Ok(())
    }
}

fn validate_path_segment(value: &str) -> Result<(), Error> {
    let mut components = Path::new(value).components();
    if value.contains('/')
        || value.contains('\\')
        || !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
    {
        return Err(Error::InvalidArticleSlug(value.to_owned()));
    }

    Ok(())
}

fn validate_relative_asset_path(path: &Path) -> Result<(), Error> {
    let mut components = path.components().peekable();
    if components.peek().is_none()
        || components.any(|component| !matches!(component, std::path::Component::Normal(_)))
    {
        return Err(Error::InvalidAssetPath(path.to_path_buf()));
    }

    Ok(())
}
