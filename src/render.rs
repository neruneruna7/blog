use askama::Template;

use crate::article::ArticleManifest;
use crate::config::SiteConfig;

/// 記事ページ描画用の入力である。
///
/// # なぜ実装するのか
/// Askamaテンプレートへ渡す値を明示し，Rust側が所有するメタ情報を一箇所に集めるためである。
///
/// # 期待する動作
/// 記事本文HTML，canonical URL，OGP画像URL，タグなどを保持する。
///
/// # 使い方の概要
/// `render_article`に渡して最終HTMLを得る。
pub struct ArticlePage<'a> {
    pub site: &'a SiteConfig,
    pub article: &'a ArticleManifest,
    pub body_html: &'a str,
    pub canonical_url: String,
    pub ogp_url: String,
}

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a> {
    page: ArticlePage<'a>,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    site: &'a SiteConfig,
    articles: Vec<&'a ArticleManifest>,
}

/// 記事ページHTMLを描画する。
///
/// # なぜ実装するのか
/// Typstが生成した本文を，サイト共通のHTML shellとメタ情報へ埋め込むためである。
///
/// # 期待する動作
/// canonical URL，OGP meta，記事metadata，本文を含むHTML文字列を返す。
///
/// # 使い方の概要
/// build処理でbody.htmlを読み込んだ後，公開記事ごとに呼び出す。
pub fn render_article(page: ArticlePage<'_>) -> askama::Result<String> {
    ArticleTemplate { page }.render()
}

/// index.htmlを描画する。
///
/// # なぜ実装するのか
/// 公開記事一覧を静的なトップページとして生成するためである。
///
/// # 期待する動作
/// draftを含まない記事カードのHTML文字列を返す。
///
/// # 使い方の概要
/// build処理で公開記事を渡して呼び出す。
pub fn render_index<'a>(
    site: &'a SiteConfig,
    articles: Vec<&'a ArticleManifest>,
) -> askama::Result<String> {
    IndexTemplate { site, articles }.render()
}

/// 記事のcanonical URLを生成する。
///
/// # なぜ実装するのか
/// 公開URLをslugから生成し，ローカルディレクトリ名への依存を排除するためである。
///
/// # 期待する動作
/// `base_url/articles/<slug>/`形式のURLを返す。
///
/// # 使い方の概要
/// HTML meta，feed，sitemap生成で利用する。
pub fn article_url(site: &SiteConfig, article: &ArticleManifest) -> String {
    format!(
        "{}/articles/{}/",
        site.base_url.trim_end_matches('/'),
        article.config.slug
    )
}

/// XML向けに最小限のエスケープを行う。
///
/// # なぜ実装するのか
/// feedとsitemapでタイトルや説明文を安全に埋め込むためである。
///
/// # 期待する動作
/// XML特殊文字を実体参照へ変換する。
///
/// # 使い方の概要
/// RSSのtext nodeと属性相当の文字列に適用する。
pub fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
