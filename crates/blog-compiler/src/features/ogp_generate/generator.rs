use std::{collections::HashMap, path::Path};

use crate::{
    commons::article::{Slug, Tag},
    features::source_parse::{ParsedSource, article::Article, site::Site},
};

use super::error::Error;

/// 記事とSiteのメタデータからOGP画像を生成する。
pub struct OgpGenerator;

impl OgpGenerator {
    /// 解析済み記事すべてのOGP PNGを生成する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use blog_compiler::{
    ///     commons::site::SiteTitle,
    ///     features::{ogp_generate::OgpGenerator, source_parse::{ParsedSource, site::Site}},
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
    /// let images = OgpGenerator::generate_all(&source, "typst-src", "ogp/main.typ")?;
    /// assert!(images.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// いずれかの記事のOGPをTypstで生成できない場合に返す。
    pub fn generate_all(
        parsed_source: &ParsedSource,
        typst_project_root: impl AsRef<Path>,
        main_file: impl AsRef<Path>,
    ) -> Result<HashMap<Slug, Vec<u8>>, Error> {
        let typst_project_root = typst_project_root.as_ref();
        let main_file = main_file.as_ref();

        parsed_source
            .articles
            .iter()
            .map(|(slug, article)| {
                tracing::debug!(slug = %slug.as_str(), "OGP画像を生成");
                let image = Self::generate_png(
                    article,
                    &parsed_source.site,
                    typst_project_root,
                    main_file,
                )?;
                Ok((slug.clone(), image))
            })
            .collect()
    }

    /// ArticleとSiteのメタデータから、一記事分のOGP PNGを生成する。
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    ///
    /// use blog_compiler::{
    ///     commons::{
    ///         article::{ArticleTitle, Description, PublishedAt, Slug, UpdatedAt},
    ///         site::SiteTitle,
    ///     },
    ///     features::{
    ///         ogp_generate::OgpGenerator,
    ///         source_parse::{article::Article, site::Site},
    ///     },
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let article = Article {
    ///     root: PathBuf::from("typst-src/articles/example"),
    ///     entry_point: PathBuf::from("typst-src/articles/example/main.typ"),
    ///     slug: Slug::new("example".to_owned())?,
    ///     title: ArticleTitle::new("Example".to_owned())?,
    ///     description: Description::new("Example article".to_owned())?,
    ///     tags: Vec::new(),
    ///     published_at: PublishedAt::new("2026-01-01T00:00:00+00:00".to_owned())?,
    ///     updated_at: UpdatedAt::new(String::new())?,
    /// };
    /// let site = Site {
    ///     title: SiteTitle::new("Example Blog".to_owned())?,
    ///     description: "Example site".to_owned(),
    ///     language: "ja".to_owned(),
    /// };
    /// let png = OgpGenerator::generate_png(
    ///     &article,
    ///     &site,
    ///     "typst-src",
    ///     "ogp/main.typ",
    /// )?;
    /// assert!(!png.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// TypstでOGP画像を生成できない場合に返す。エラーには対象記事のSlugが含まれる。
    pub fn generate_png(
        article: &Article,
        site: &Site,
        typst_project_root: impl AsRef<Path>,
        main_file: impl AsRef<Path>,
    ) -> Result<Vec<u8>, Error> {
        let input = typst_html_export::OgpInput {
            site_title: site.title.as_str().to_owned(),
            article_title: article.title.as_str().to_owned(),
            description: article.description.as_str().to_owned(),
            tags: article
                .tags
                .iter()
                .map(Tag::as_str)
                .map(str::to_owned)
                .collect(),
        };

        typst_html_export::compile_ogp_png(typst_project_root, main_file, &input).map_err(
            |source| Error::Typst {
                slug: article.slug.as_str().to_owned(),
                source,
            },
        )
    }
}
