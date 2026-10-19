use std::{collections::HashSet, fs, path::Path};

use serde::Serialize;

use crate::{
    commons::{
        article::{ArticleTitle, Description, PublishedAt, Slug, Tag, UpdatedAt},
        atomic_file::write_atomic,
        site::SiteTitle,
    },
    features::source_parse::{ParsedSource, article::Article, site::Site},
};

use super::error::Error;

const ARTICLES_DIRECTORY: &str = "articles";
const ARTICLE_METADATA_FILE: &str = "meta.json";
const ARTICLE_INDEX_FILE: &str = "article-index.json";
const SITE_CONFIG_FILE: &str = "site.json";
const OGP_IMAGE_DIRECTORY: &str = "ogp/articles";
const OGP_IMAGE_MIME_TYPE: &str = "image/png";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SiteMetadata<'a> {
    title: &'a SiteTitle,
    description: &'a str,
    language: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ArticleMetadata<'a> {
    slug: &'a Slug,
    title: &'a ArticleTitle,
    description: &'a Description,
    published_at: &'a PublishedAt,
    updated_at: &'a UpdatedAt,
    tags: Vec<&'a str>,
    url: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct OgpImageMetadata {
    path: String,
    width: u32,
    height: u32,
    mime_type: &'static str,
    alt: String,
}

#[derive(Serialize)]
struct ArticleIndexEntry<'a> {
    #[serde(flatten)]
    article: ArticleMetadata<'a>,
    #[serde(rename = "ogpImage")]
    ogp_image: OgpImageMetadata,
}

/// `ParsedSource` からサイト・記事メタデータを生成するWriter。
pub struct MetadataWriter;

impl MetadataWriter {
    /// サイト設定と、指定されたslugの記事一覧・記事ごとのメタデータを生成する。
    ///
    /// OGP画像の公開パスを含め、指定されたslugの記事だけのメタデータを`ParsedSource`から導出する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::{HashMap, HashSet};
    ///
    /// use blog_compiler::{
    ///     commons::site::SiteTitle,
    ///     features::{metadata_write::MetadataWriter, source_parse::{ParsedSource, site::Site}},
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let source = ParsedSource {
    ///     articles: HashMap::new(),
    ///     site: Site {
    ///         title: SiteTitle::new("Example Blog".to_owned())?,
    ///         description: "Example site".to_owned(),
    ///         language: "ja".to_owned(),
    ///     },
    /// };
    /// let output_root = tempfile::tempdir()?;
    /// MetadataWriter::write(&source, &HashSet::new(), output_root.path())?;
    /// assert!(output_root.path().join("site.json").is_file());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// メタデータを直列化できない場合、または出力ファイルを置き換えられない場合に返す。
    pub fn write(
        parsed_source: &ParsedSource,
        current_slugs: &HashSet<Slug>,
        output_root: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let output_root = output_root.as_ref();
        let site_metadata = SiteMetadata {
            title: &parsed_source.site.title,
            description: &parsed_source.site.description,
            language: &parsed_source.site.language,
        };
        write_json(&output_root.join(SITE_CONFIG_FILE), &site_metadata)?;

        let mut articles = parsed_source
            .articles
            .iter()
            .filter(|(slug, _)| current_slugs.contains(*slug))
            .map(|(_, article)| article)
            .collect::<Vec<_>>();
        articles.sort_by(|left, right| left.slug.as_str().cmp(right.slug.as_str()));
        let entries = articles
            .into_iter()
            .map(|article| article_index_entry(article, &parsed_source.site))
            .collect::<Vec<_>>();
        write_json(&output_root.join(ARTICLE_INDEX_FILE), &entries)?;

        for entry in &entries {
            let path = output_root
                .join(ARTICLES_DIRECTORY)
                .join(entry.article.slug.as_str())
                .join(ARTICLE_METADATA_FILE);
            write_json(&path, entry)?;
        }

        Ok(())
    }

    /// `current_slugs`から外れた記事の`meta.json`を削除する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use blog_compiler::{commons::article::Slug, features::metadata_write::MetadataWriter};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output_root = tempfile::tempdir()?;
    /// let current_slugs = HashSet::<Slug>::new();
    /// MetadataWriter::remove_obsolete(&current_slugs, output_root.path())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 記事ディレクトリを検査できない場合、または不要なメタデータを削除できない場合に返す。
    pub fn remove_obsolete(
        current_slugs: &HashSet<Slug>,
        output_root: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let directory = output_root.as_ref().join(ARTICLES_DIRECTORY);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(Error::ReadArticleDirectory {
                    path: directory,
                    source,
                });
            }
        };
        let current_slug_names = current_slugs
            .iter()
            .map(Slug::as_str)
            .collect::<HashSet<_>>();

        for entry in entries {
            let entry = entry.map_err(|source| Error::ReadArticleDirectory {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            if !entry
                .file_type()
                .map_err(|source| Error::InspectArticle {
                    path: path.clone(),
                    source,
                })?
                .is_dir()
            {
                continue;
            }

            let slug = entry.file_name().to_string_lossy().into_owned();
            if current_slug_names.contains(slug.as_str()) {
                continue;
            }

            let metadata_path = path.join(ARTICLE_METADATA_FILE);
            match fs::remove_file(&metadata_path) {
                Ok(()) => {}
                Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
                Err(source) => {
                    return Err(Error::RemoveObsoleteMetadata {
                        path: metadata_path,
                        source,
                    });
                }
            }
            match fs::remove_dir(&path) {
                Ok(()) => {}
                Err(source)
                    if matches!(
                        source.kind(),
                        std::io::ErrorKind::NotFound | std::io::ErrorKind::DirectoryNotEmpty
                    ) => {}
                Err(source) => {
                    return Err(Error::RemoveEmptyDirectory { path, source });
                }
            }
        }

        Ok(())
    }
}

fn article_index_entry<'a>(article: &'a Article, site: &Site) -> ArticleIndexEntry<'a> {
    ArticleIndexEntry {
        article: ArticleMetadata {
            slug: &article.slug,
            title: &article.title,
            description: &article.description,
            published_at: &article.published_at,
            updated_at: &article.updated_at,
            tags: article.tags.iter().map(Tag::as_str).collect(),
            url: format!("/articles/{}/", article.slug.as_str()),
        },
        ogp_image: OgpImageMetadata {
            path: format!("/{OGP_IMAGE_DIRECTORY}/{}.png", article.slug.as_str()),
            width: typst_html_export::OGP_WIDTH_PX,
            height: typst_html_export::OGP_HEIGHT_PX,
            mime_type: OGP_IMAGE_MIME_TYPE,
            alt: format!("{} | {}", article.title.as_str(), site.title.as_str()),
        },
    }
}

fn write_json(path: &Path, value: &(impl Serialize + ?Sized)) -> Result<(), Error> {
    let mut contents = serde_json::to_vec_pretty(value).map_err(|source| Error::SerializeJson {
        path: path.to_path_buf(),
        source,
    })?;
    contents.push(b'\n');
    write_atomic(path, &contents).map_err(|source| Error::WriteJson {
        path: path.to_path_buf(),
        source,
    })
}
