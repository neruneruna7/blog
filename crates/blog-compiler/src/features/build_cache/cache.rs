use std::collections::HashSet;

use crate::{commons::article::Slug, features::blog_cache_loader::BlogCacheInput};

/// 現在の入力と前回の指紋を比較し、HTML・OGPそれぞれの再生成要否を示す。
#[derive(Debug, Clone)]
pub struct BuildCache {
    article_count: usize,
    cached_html: HashSet<Slug>,
    cached_ogp: HashSet<Slug>,
}

impl BuildCache {
    /// キャッシュ入力から、成果物ごとの再生成判定を作る。
    ///
    /// この判定は入力値だけを参照し、ファイルの読み書きを行わない。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use blog_compiler::{
    ///     commons::site::SiteTitle,
    ///     features::{
    ///         blog_cache_loader::BlogCacheLoader,
    ///         build_cache::BuildCache,
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
    /// let input = BlogCacheLoader::load(&source, source_root.path(), output_root.path())?;
    /// let plan = BuildCache::new(&input);
    /// assert_eq!(plan.html_count(), 0);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(input: &BlogCacheInput) -> Self {
        let mut cached_html = HashSet::new();
        let mut cached_ogp = HashSet::new();

        for (slug, article_input) in &input.articles {
            if input
                .cached_articles
                .get(slug.as_str())
                .and_then(|entry| entry.html_hash.as_ref())
                == Some(&article_input.html_hash)
                && article_input.html_is_readable
            {
                cached_html.insert(slug.clone());
            }
            if input
                .cached_articles
                .get(slug.as_str())
                .and_then(|entry| entry.ogp_hash.as_ref())
                == Some(&article_input.ogp_hash)
                && article_input.ogp_is_valid
            {
                cached_ogp.insert(slug.clone());
            }
        }

        Self {
            article_count: input.articles.len(),
            cached_html,
            cached_ogp,
        }
    }

    /// 指定記事のHTMLを変換する必要があるかを返す。
    pub fn needs_html(&self, slug: &Slug) -> bool {
        !self.cached_html.contains(slug)
    }

    /// 指定記事のOGP画像を生成する必要があるかを返す。
    pub fn needs_ogp(&self, slug: &Slug) -> bool {
        !self.cached_ogp.contains(slug)
    }

    /// HTML変換が必要な記事数を返す。
    pub fn html_count(&self) -> usize {
        self.article_count - self.cached_html.len()
    }

    /// OGP生成が必要な記事数を返す。
    pub fn ogp_count(&self) -> usize {
        self.article_count - self.cached_ogp.len()
    }
}
