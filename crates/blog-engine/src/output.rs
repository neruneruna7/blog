//! なぜ実装するのか:
//! - 検証済み記事ドメインモデルからJSON出力モデルへの変換と書き出しを集約するため。
//!
//! 期待する動作:
//! - サイト、記事一覧、記事別メタデータを既存契約のpretty JSONで生成する。
//!
//! 使い方の概要:
//! - ビルド処理の後段で `write_site_output` と `write_article_outputs` を呼び出す。

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::article::Article;
use crate::error::BlogEngineError;

/// なぜ実装するのか:
/// - `generated/site.json` の安定したJSON契約をRust型で表すため。
///
/// 期待する動作:
/// - `base_url` を `baseUrl` として出力し、その他のサイトメタデータを保持する。
///
/// 使い方の概要:
/// - `config::load_site_config` の戻り値として受け取り、`write_site_output` へ渡す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteOutput {
    pub title: String,
    pub description: String,
    pub language: String,
    pub base_url: String,
}

/// なぜ実装するのか:
/// - 検証済み記事ドメインモデルを、既存のJSON外部契約へ変換するため。
///
/// 期待する動作:
/// - 日時をRFC3339文字列としてcamelCaseのJSONキーで出力する。
/// - 既存契約の `ogpImage` をnullで維持し、この型で画像生成や画像検証を行わない。
///
/// 使い方の概要:
/// - `Article` から変換し、`write_article_outputs` でファイルへ書き出す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleOutput {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub updated_at: Option<String>,
    pub tags: Vec<String>,
    pub url: String,
    pub ogp_image: Option<String>,
}

impl From<&Article> for ArticleOutput {
    /// なぜ実装するのか:
    /// - ドメインモデルとJSON出力モデルの変換を出力境界へ集約するため。
    ///
    /// 期待する動作:
    /// - `DateTime<FixedOffset>` をRFC3339文字列へ変換し、更新日時のNoneをnullへ対応させる。
    ///
    /// 使い方の概要:
    /// - `ArticleOutput::from(&article)` として呼び出す。
    fn from(article: &Article) -> Self {
        Self {
            id: article.id.clone(),
            slug: article.slug.clone(),
            title: article.title.clone(),
            description: article.description.clone(),
            published_at: article.published_at.to_rfc3339(),
            updated_at: article.updated_at.as_ref().map(|date| date.to_rfc3339()),
            tags: article.tags.clone(),
            url: article.url.clone(),
            ogp_image: None,
        }
    }
}

/// なぜ実装するのか:
/// - サイトメタデータの生成ファイルを書き出すため。
///
/// 期待する動作:
/// - 出力ディレクトリを作成し、pretty JSONで `site.json` を書く。
///
/// 使い方の概要:
/// - ビルド処理から出力ルートと `SiteOutput` を渡して呼び出す。
pub fn write_site_output(
    output_root: impl AsRef<Path>,
    site: &SiteOutput,
) -> Result<(), BlogEngineError> {
    let output_root = output_root.as_ref();
    create_dir_all(output_root)?;
    write_json(output_root.join("site.json"), site)
}

/// なぜ実装するのか:
/// - 記事インデックスと記事別メタデータを同じ契約で書き出すため。
///
/// 期待する動作:
/// - `article-index.json` と `articles/<slug>/meta.json` をpretty JSONで生成する。
/// - 出力ディレクトリ名には、ソース記事ディレクトリ名ではなく設定済みslugを使用する。
///
/// 使い方の概要:
/// - 検証済み `Article` の一覧を渡し、内部で `ArticleOutput` へ変換して呼び出す。
pub fn write_article_outputs(
    output_root: impl AsRef<Path>,
    articles: &[Article],
) -> Result<(), BlogEngineError> {
    let output_root = output_root.as_ref();
    create_dir_all(output_root)?;
    let article_outputs = articles.iter().map(ArticleOutput::from).collect::<Vec<_>>();
    write_json(output_root.join("article-index.json"), &article_outputs)?;

    for article in articles {
        let article_dir = output_root.join("articles").join(&article.slug);
        create_dir_all(&article_dir)?;
        let output = ArticleOutput::from(article);
        write_json(article_dir.join("meta.json"), &output)?;
    }

    Ok(())
}

fn create_dir_all(path: &Path) -> Result<(), BlogEngineError> {
    fs::create_dir_all(path).map_err(|source| BlogEngineError::io(path, source))
}

fn write_json(
    path: impl AsRef<Path>,
    value: &(impl Serialize + ?Sized),
) -> Result<(), BlogEngineError> {
    let path = path.as_ref();
    let content = serde_json::to_string_pretty(value)
        .expect("serializing typed metadata output to JSON must not fail");
    fs::write(path, format!("{content}\n")).map_err(|source| BlogEngineError::io(path, source))
}
