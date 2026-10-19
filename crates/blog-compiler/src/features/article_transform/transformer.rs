use std::collections::HashMap;

use crate::{commons::article::Slug, features::source_parse::article::Article};

use super::{error::Error, postprocess::Pipeline};

pub struct ArticleTransformer;

impl ArticleTransformer {
    /// 再生成対象の記事を HTML に変換し、公開用の本文 HTML を返す。
    ///
    /// 入力 HashMap のキーと各 Article の Slug が一致している必要がある。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use blog_compiler::{
    ///     commons::article::Slug,
    ///     features::{article_transform::ArticleTransformer, source_parse::article::Article},
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let html_by_slug = ArticleTransformer::transform_all(HashMap::<Slug, Article>::new())?;
    /// assert!(html_by_slug.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Typst の HTML 変換や後処理に失敗した場合、または HashMap のキーと Article の Slug が一致しない場合に返す。
    pub fn transform_all(articles: HashMap<Slug, Article>) -> Result<HashMap<Slug, String>, Error> {
        if articles.is_empty() {
            return Ok(HashMap::new());
        }
        let postprocess = Pipeline::production();
        articles
            .into_iter()
            .map(|(slug, article)| {
                if slug != article.slug {
                    return Err(Error::ArticleSlugMismatch {
                        key_slug: slug.as_str().to_owned(),
                        article_slug: article.slug.as_str().to_owned(),
                    });
                }
                Self::transform_article(&article, &postprocess).map(|html| (slug, html))
            })
            .collect()
    }

    fn transform_article(article: &Article, postprocess: &Pipeline) -> Result<String, Error> {
        tracing::debug!(slug = %article.slug.as_str(), "記事をHTMLへ変換");
        let html = typst_html_export::compile_html(&article.root, &article.entry_point).map_err(
            |source| Error::TypstHtml {
                slug: article.slug.as_str().to_owned(),
                source,
            },
        )?;
        let postprocessed = postprocess.process(html, article.slug.as_str())?;
        for warning in &postprocessed.warnings {
            tracing::warn!(
                slug = %article.slug.as_str(),
                url = ?warning.url,
                reason = ?warning.reason,
                "記事の後処理で警告"
            );
        }

        Ok(postprocessed.body_html)
    }
}
