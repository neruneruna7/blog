use crate::features::source_parse::article::Article;

/// ArticleConfigで公開対象に指定された記事かを判定する。
pub struct PublicationFilter;

impl PublicationFilter {
    /// 記事が公開対象に指定されているかを返す。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::path::PathBuf;
    ///
    /// use blog_compiler::{
    ///     commons::article::{ArticleTitle, Description, PublishedAt, Slug, UpdatedAt},
    ///     features::{publication_filter::PublicationFilter, source_parse::article::Article},
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let article = Article {
    ///     root: PathBuf::from("typst-src/articles/example"),
    ///     entry_point: PathBuf::from("typst-src/articles/example/main.typ"),
    ///     slug: Slug::new("example".to_owned())?,
    ///     publish: false,
    ///     title: ArticleTitle::new("Example".to_owned())?,
    ///     description: Description::new("Example article".to_owned())?,
    ///     tags: Vec::new(),
    ///     published_at: PublishedAt::new("2026-01-01T00:00:00+00:00".to_owned())?,
    ///     updated_at: UpdatedAt::new(String::new())?,
    /// };
    /// assert!(!PublicationFilter::is_publication_target(&article));
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_publication_target(article: &Article) -> bool {
        article.publish
    }
}
