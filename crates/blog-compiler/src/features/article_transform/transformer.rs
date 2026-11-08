use std::{collections::HashMap, path::Path};

use crate::{
    commons::article::Slug,
    features::{source_parse::article::Article, web_link_card_ogp_input::WebLinkCardOgpLoader},
};

use super::{
    error::Error,
    postprocess::{self, Pipeline},
};

pub struct ArticleTransformer;

impl ArticleTransformer {
    /// 再生成対象の記事を HTML に変換し、公開用の本文 HTML を返す。
    ///
    /// 入力 HashMap のキーと各 Article の Slug が一致している必要がある。キャッシュrootを渡すと、
    /// リンクカードのOGP情報を`<root>/.cache/web-link-card`へ保存・再利用する。
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
    /// let html_by_slug = ArticleTransformer::transform_all(HashMap::<Slug, Article>::new(), None)?;
    /// assert!(html_by_slug.is_empty());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Typst の HTML 変換や後処理に失敗した場合、または HashMap のキーと Article の Slug が一致しない場合に返す。
    pub fn transform_all(
        articles: HashMap<Slug, Article>,
        cache_root: Option<&Path>,
    ) -> Result<HashMap<Slug, String>, Error> {
        if articles.is_empty() {
            return Ok(HashMap::new());
        }
        let compiled_articles = articles
            .into_iter()
            .map(|(slug, article)| {
                if slug != article.slug {
                    return Err(Error::ArticleSlugMismatch {
                        key_slug: slug.as_str().to_owned(),
                        article_slug: article.slug.as_str().to_owned(),
                    });
                }
                Self::compile_article(&article).map(|html| (slug, html))
            })
            .collect::<Result<HashMap<_, _>, _>>()?;

        let urls = compiled_articles
            .iter()
            .map(|(slug, html)| postprocess::collect_web_link_card_urls(html, slug.as_str()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let web_link_card_input = WebLinkCardOgpLoader::load(&urls, cache_root);
        let postprocess = Pipeline::new();

        compiled_articles
            .into_iter()
            .map(|(slug, html)| {
                postprocess
                    .process(html, slug.as_str(), &web_link_card_input)
                    .map(|output| {
                        for warning in &output.warnings {
                            tracing::warn!(
                                slug = %slug.as_str(),
                                url = ?warning.url,
                                reason = ?warning.reason,
                                "記事の後処理で警告"
                            );
                        }
                        (slug, output.body_html)
                    })
            })
            .collect()
    }

    fn compile_article(article: &Article) -> Result<String, Error> {
        tracing::debug!(slug = %article.slug.as_str(), "記事をHTMLへ変換");
        typst_html_export::compile_html(&article.root, &article.entry_point).map_err(|source| {
            Error::TypstHtml {
                slug: article.slug.as_str().to_owned(),
                source,
            }
        })
    }
}
