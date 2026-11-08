use std::{
    collections::{BTreeMap, HashMap, HashSet},
    path::{Path, PathBuf},
};

use crate::{
    commons::{
        article::Slug,
        atomic_file::write_atomic,
        build_cache_manifest::{BuildCacheManifest, MANIFEST_SCHEMA_VERSION, ManifestArticle},
    },
    features::blog_cache_input::BlogCacheInput,
};

use super::error::Error;

const CACHE_DIRECTORY: &str = ".cache";
const MANIFEST_FILE: &str = "manifest.json";

#[derive(Debug, Clone)]
struct Fingerprint {
    html_hash: String,
    ogp_hash: String,
}

/// 成果物の書き込み成功を単一のキャッシュManifestへ反映する。
///
/// 各成果物のWriterが成功した後に対応する記録を呼び出す。Manifestは更新ごとに原子的に置き換わる。
#[derive(Debug, Clone)]
pub struct BuildCacheWriter {
    manifest_path: PathBuf,
    manifest: BuildCacheManifest,
    manifest_needs_rewrite: bool,
    fingerprints: HashMap<Slug, Fingerprint>,
}

impl BuildCacheWriter {
    /// キャッシュ入力の履歴を引き継ぎ、Manifestの更新先を準備する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::{HashMap, HashSet};
    ///
    /// use blog_compiler::{
    ///     commons::site::SiteTitle,
    ///     features::{
    ///         blog_cache_input::BlogCacheLoader,
    ///         build_cache_output::BuildCacheWriter,
    ///         source_parse::{ParsedSource, site::Site},
    ///     },
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
    /// let input = BlogCacheLoader::load(
    ///     &source,
    ///     &HashSet::new(),
    ///     source_root.path(),
    ///     output_root.path(),
    /// )?;
    /// let writer = BuildCacheWriter::new(input, output_root.path());
    /// let _ = writer;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(input: BlogCacheInput, output_root: impl AsRef<Path>) -> Self {
        let manifest = BuildCacheManifest {
            schema_version: MANIFEST_SCHEMA_VERSION,
            articles: input
                .cached_articles
                .into_iter()
                .map(|(slug, article)| {
                    (
                        slug,
                        ManifestArticle {
                            html_hash: article.html_hash,
                            ogp_hash: article.ogp_hash,
                        },
                    )
                })
                .collect::<BTreeMap<_, _>>(),
        };
        let fingerprints = input
            .articles
            .into_iter()
            .map(|(slug, article)| {
                (
                    slug,
                    Fingerprint {
                        html_hash: article.html_hash,
                        ogp_hash: article.ogp_hash,
                    },
                )
            })
            .collect::<HashMap<_, _>>();
        Self {
            manifest_path: output_root
                .as_ref()
                .join(CACHE_DIRECTORY)
                .join(MANIFEST_FILE),
            manifest,
            manifest_needs_rewrite: input.manifest_needs_rewrite,
            fingerprints,
        }
    }

    /// HTMLの書き込み成功を、その記事の現在の入力指紋とともに記録する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::{HashMap, HashSet};
    ///
    /// use blog_compiler::{
    ///     commons::{article::Slug, site::SiteTitle},
    ///     features::{
    ///         blog_cache_input::BlogCacheLoader,
    ///         build_cache_output::BuildCacheWriter,
    ///         source_parse::{ParsedSource, site::Site},
    ///     },
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
    /// let input = BlogCacheLoader::load(
    ///     &source,
    ///     &HashSet::new(),
    ///     source_root.path(),
    ///     output_root.path(),
    /// )?;
    /// let mut writer = BuildCacheWriter::new(input, output_root.path());
    /// let slug = Slug::new("missing-article".to_owned())?;
    /// assert!(writer.record_html_success(&slug).is_err());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 今回の入力に存在しないslugが指定された場合、またはManifestを原子的に置換できない場合に返す。
    pub fn record_html_success(&mut self, slug: &Slug) -> Result<(), Error> {
        let hash = self
            .fingerprints
            .get(slug)
            .ok_or_else(|| Error::UnknownArticleSlug(slug.as_str().to_owned()))?
            .html_hash
            .clone();
        self.update_hash(slug, hash, true)
    }

    /// OGP画像の書き込み成功を、その記事の現在の入力指紋とともに記録する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::{HashMap, HashSet};
    ///
    /// use blog_compiler::{
    ///     commons::{article::Slug, site::SiteTitle},
    ///     features::{
    ///         blog_cache_input::BlogCacheLoader,
    ///         build_cache_output::BuildCacheWriter,
    ///         source_parse::{ParsedSource, site::Site},
    ///     },
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
    /// let input = BlogCacheLoader::load(
    ///     &source,
    ///     &HashSet::new(),
    ///     source_root.path(),
    ///     output_root.path(),
    /// )?;
    /// let mut writer = BuildCacheWriter::new(input, output_root.path());
    /// let slug = Slug::new("missing-article".to_owned())?;
    /// assert!(writer.record_ogp_success(&slug).is_err());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 今回の入力に存在しないslugが指定された場合、またはManifestを原子的に置換できない場合に返す。
    pub fn record_ogp_success(&mut self, slug: &Slug) -> Result<(), Error> {
        let hash = self
            .fingerprints
            .get(slug)
            .ok_or_else(|| Error::UnknownArticleSlug(slug.as_str().to_owned()))?
            .ogp_hash
            .clone();
        self.update_hash(slug, hash, false)
    }

    /// 今回の入力から削除された記事の指紋をManifestから取り除く。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::{HashMap, HashSet};
    ///
    /// use blog_compiler::{
    ///     commons::site::SiteTitle,
    ///     features::{
    ///         blog_cache_input::BlogCacheLoader,
    ///         build_cache_output::BuildCacheWriter,
    ///         source_parse::{ParsedSource, site::Site},
    ///     },
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
    /// let input = BlogCacheLoader::load(
    ///     &source,
    ///     &HashSet::new(),
    ///     source_root.path(),
    ///     output_root.path(),
    /// )?;
    /// let mut writer = BuildCacheWriter::new(input, output_root.path());
    /// writer.remove_obsolete(&HashSet::new())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Manifestを原子的に置換できない場合に返す。
    pub fn remove_obsolete(&mut self, current_slugs: &HashSet<Slug>) -> Result<(), Error> {
        let mut next_manifest = self.manifest.clone();
        let previous_count = next_manifest.articles.len();
        let current_slug_names = current_slugs
            .iter()
            .map(|slug| slug.as_str())
            .collect::<HashSet<_>>();
        next_manifest
            .articles
            .retain(|slug, _| current_slug_names.contains(slug.as_str()));
        if self.manifest_needs_rewrite || next_manifest.articles.len() != previous_count {
            self.persist(next_manifest)?;
        }
        Ok(())
    }

    fn update_hash(&mut self, slug: &Slug, hash: String, is_html: bool) -> Result<(), Error> {
        let mut next_manifest = self.manifest.clone();
        let entry = next_manifest
            .articles
            .entry(slug.as_str().to_owned())
            .or_insert_with(ManifestArticle::empty);
        let target_hash = if is_html {
            &mut entry.html_hash
        } else {
            &mut entry.ogp_hash
        };
        if target_hash.as_ref() == Some(&hash) {
            return Ok(());
        }
        *target_hash = Some(hash);
        self.persist(next_manifest)
    }

    fn persist(&mut self, manifest: BuildCacheManifest) -> Result<(), Error> {
        let mut contents =
            serde_json::to_vec_pretty(&manifest).map_err(Error::SerializeManifest)?;
        contents.push(b'\n');
        write_atomic(&self.manifest_path, &contents).map_err(|source| Error::WriteManifest {
            path: self.manifest_path.clone(),
            source,
        })?;
        self.manifest = manifest;
        self.manifest_needs_rewrite = false;
        Ok(())
    }
}
