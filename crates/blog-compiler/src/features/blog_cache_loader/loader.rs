use std::{
    collections::{BTreeMap, HashMap},
    fs::{self, File},
    path::Path,
};

use tracing::warn;

use crate::{
    commons::{
        article::Slug,
        build_cache_manifest::{BuildCacheManifest, MANIFEST_SCHEMA_VERSION},
    },
    features::source_parse::ParsedSource,
};

use super::{
    error::Error,
    hash::{html_hash, ogp_hash},
};

const ARTICLES_DIRECTORY: &str = "articles";
const BODY_HTML_FILE: &str = "body.html";
const PUBLIC_ROOT_DIRECTORY: &str = "public-root";
const OGP_ARTICLES_DIRECTORY: &str = "ogp/articles";
const PNG_SIGNATURE: &[u8] = b"\x89PNG\r\n\x1a\n";
const CACHE_DIRECTORY: &str = ".cache";
const MANIFEST_FILE: &str = "manifest.json";

/// 現在指紋と、既存HTML・OGP成果物が再利用可能かを表す。
#[derive(Debug, Clone)]
pub(crate) struct ArticleCacheInput {
    pub(crate) html_hash: String,
    pub(crate) ogp_hash: String,
    pub(crate) html_is_readable: bool,
    pub(crate) ogp_is_valid: bool,
}

/// 前回のビルドで成果物の書き込みに成功した入力指紋。
#[derive(Debug, Clone)]
pub(crate) struct CachedArticleFingerprints {
    pub(crate) html_hash: Option<String>,
    pub(crate) ogp_hash: Option<String>,
}

/// 再生成判定に使う現在値と前回成功した指紋をまとめた入力。
#[derive(Debug, Clone)]
pub struct BlogCacheInput {
    pub(crate) articles: HashMap<Slug, ArticleCacheInput>,
    pub(crate) cached_articles: BTreeMap<String, CachedArticleFingerprints>,
    pub(crate) manifest_needs_rewrite: bool,
}

/// 現在の入力指紋、既存成果物の利用可否、前回成功した指紋を集める。
pub struct BlogCacheLoader;

impl BlogCacheLoader {
    /// `ParsedSource`と生成先から、再生成判定とManifest更新に必要な入力を返す。
    ///
    /// キャッシュManifestの読み込みに失敗した場合は空の履歴として扱い、全成果物を再生成できる状態を返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use blog_compiler::{
    ///     commons::site::SiteTitle,
    ///     features::source_parse::{ParsedSource, site::Site},
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
    /// let source_root = tempfile::tempdir()?;
    /// let output_root = tempfile::tempdir()?;
    /// let input = BlogCacheLoader::load(&source, source_root.path(), output_root.path())?;
    /// let _input = input;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// ソースツリーの走査・読み込み、またはOGP指紋入力の直列化に失敗した場合に返す。
    pub fn load(
        parsed_source: &ParsedSource,
        source_root: impl AsRef<Path>,
        output_root: impl AsRef<Path>,
    ) -> Result<BlogCacheInput, Error> {
        let source_root = source_root.as_ref();
        let output_root = output_root.as_ref();
        let mut articles = HashMap::with_capacity(parsed_source.articles.len());

        for (slug, article) in &parsed_source.articles {
            let article_input = ArticleCacheInput {
                html_hash: html_hash(article, source_root)?,
                ogp_hash: ogp_hash(article, &parsed_source.site, source_root)?,
                html_is_readable: cached_html_is_readable(output_root, slug),
                ogp_is_valid: cached_ogp_is_valid(output_root, slug),
            };
            articles.insert(slug.clone(), article_input);
        }

        let (manifest, manifest_needs_rewrite) =
            read_manifest(&output_root.join(CACHE_DIRECTORY).join(MANIFEST_FILE));
        let cached_articles = manifest
            .articles
            .into_iter()
            .map(|(slug, entry)| {
                (
                    slug,
                    CachedArticleFingerprints {
                        html_hash: entry.html_hash,
                        ogp_hash: entry.ogp_hash,
                    },
                )
            })
            .collect();

        Ok(BlogCacheInput {
            articles,
            cached_articles,
            manifest_needs_rewrite,
        })
    }
}

fn read_manifest(path: &Path) -> (BuildCacheManifest, bool) {
    let contents = match fs::read(path) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return (BuildCacheManifest::empty(), false);
        }
        Err(error) => {
            warn!(path = %path.display(), %error, "キャッシュManifestを読み取れないため再生成");
            return (BuildCacheManifest::empty(), true);
        }
    };

    match serde_json::from_slice::<BuildCacheManifest>(&contents) {
        Ok(manifest) if manifest.schema_version == MANIFEST_SCHEMA_VERSION => (manifest, false),
        Ok(_) => {
            warn!(path = %path.display(), "キャッシュManifestの形式が古いため再生成");
            (BuildCacheManifest::empty(), true)
        }
        Err(error) => {
            warn!(path = %path.display(), %error, "キャッシュManifestを読み取れないため再生成");
            (BuildCacheManifest::empty(), true)
        }
    }
}

fn cached_html_is_readable(output_root: &Path, slug: &Slug) -> bool {
    let path = output_root
        .join(ARTICLES_DIRECTORY)
        .join(slug.as_str())
        .join(BODY_HTML_FILE);
    fs::metadata(&path).is_ok_and(|metadata| metadata.is_file()) && File::open(path).is_ok()
}

fn cached_ogp_is_valid(output_root: &Path, slug: &Slug) -> bool {
    let path = output_root
        .join(PUBLIC_ROOT_DIRECTORY)
        .join(OGP_ARTICLES_DIRECTORY)
        .join(format!("{}.png", slug.as_str()));
    fs::read(path)
        .ok()
        .is_some_and(|image| valid_ogp_png(&image))
}

fn valid_ogp_png(image: &[u8]) -> bool {
    if !image.starts_with(PNG_SIGNATURE) {
        return false;
    }

    let mut offset = PNG_SIGNATURE.len();
    let mut is_first_chunk = true;
    while let Some(chunk_header_end) = offset.checked_add(8) {
        if chunk_header_end > image.len() {
            return false;
        }
        let chunk_length = u32::from_be_bytes([
            image[offset],
            image[offset + 1],
            image[offset + 2],
            image[offset + 3],
        ]) as usize;
        let Some(chunk_end) = chunk_header_end
            .checked_add(chunk_length)
            .and_then(|end| end.checked_add(4))
        else {
            return false;
        };
        if chunk_end > image.len() {
            return false;
        }

        let chunk_type = &image[offset + 4..chunk_header_end];
        if is_first_chunk {
            let valid_header = chunk_type == b"IHDR"
                && chunk_length == 13
                && u32::from_be_bytes(image[offset + 8..offset + 12].try_into().unwrap())
                    == typst_html_export::OGP_WIDTH_PX
                && u32::from_be_bytes(image[offset + 12..offset + 16].try_into().unwrap())
                    == typst_html_export::OGP_HEIGHT_PX;
            if !valid_header {
                return false;
            }
            is_first_chunk = false;
        }
        if chunk_type == b"IEND" {
            return chunk_length == 0 && chunk_end == image.len();
        }
        offset = chunk_end;
    }
    false
}
