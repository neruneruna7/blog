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
use crate::ogp::OgpImageMeta;

/// なぜ実装するのか:
/// - `generated/site.json` の安定したJSON契約をRust型で表すため。
///
/// 期待する動作:
/// - 記事コンパイルに必要なサイト表示メタデータを保持する。
///
/// 使い方の概要:
/// - `config::load_site_config` の戻り値として受け取り、`write_site_output` へ渡す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SiteOutput {
    pub title: String,
    pub description: String,
    pub language: String,
}

/// なぜ実装するのか:
/// - 検証済み記事ドメインモデルを、既存のJSON外部契約へ変換するため。
///
/// 期待する動作:
/// - 日時をRFC3339文字列としてcamelCaseのJSONキーで出力する。
/// - 成功済みOGP成果物の構造化メタデータを必須値として保持する。
///
/// 使い方の概要:
/// - `Article` と `OgpImageMeta` から構築し、`write_article_outputs` で書き出す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleOutput {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub updated_at: Option<String>,
    pub tags: Vec<String>,
    pub url: String,
    pub ogp_image: OgpImageMeta,
}

impl ArticleOutput {
    /// なぜ実装するのか:
    /// - ドメインモデルと成功済みOGP成果物を、最終JSONの一つの契約へ結合するため。
    ///
    /// 期待する動作:
    /// - `DateTime<FixedOffset>` をRFC3339文字列へ変換する。
    /// - OGPメタデータを省略せず、そのまま記事JSONへ保持する。
    ///
    /// 使い方の概要:
    /// - OGP生成が成功した後に `ArticleOutput::from_article` を呼び出す。
    pub fn from_article(article: &Article, ogp_image: OgpImageMeta) -> Self {
        Self {
            slug: article.slug.clone(),
            title: article.title.clone(),
            description: article.description.clone(),
            published_at: article.published_at.to_rfc3339(),
            updated_at: article.updated_at.as_ref().map(|date| date.to_rfc3339()),
            tags: article.tags.clone(),
            url: article.url.clone(),
            ogp_image,
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
/// - 記事インデックスと記事別メタデータを、同じOGP付き契約で書き出すため。
///
/// 期待する動作:
/// - `article-index.json` と `articles/<slug>/meta.json` をpretty JSONで生成する。
/// - 出力ディレクトリ名には、ソース記事ディレクトリ名ではなく設定済みslugを使用する。
/// - 記事とOGPメタデータの件数が一致しない場合は書き出さない。
///
/// 使い方の概要:
/// - 検証済み `Article` の一覧を渡し、内部で `ArticleOutput` へ変換して呼び出す。
pub fn write_article_outputs(
    output_root: impl AsRef<Path>,
    articles: &[Article],
    ogp_images: &[OgpImageMeta],
) -> Result<(), BlogEngineError> {
    let output_root = output_root.as_ref();
    if articles.len() != ogp_images.len() {
        return Err(BlogEngineError::validation(format!(
            "article and OGP metadata counts must match: articles={}, ogp_images={}",
            articles.len(),
            ogp_images.len()
        )));
    }

    create_dir_all(output_root)?;
    let article_outputs = articles
        .iter()
        .zip(ogp_images)
        .map(|(article, ogp_image)| ArticleOutput::from_article(article, ogp_image.clone()))
        .collect::<Vec<_>>();
    write_json(output_root.join("article-index.json"), &article_outputs)?;

    for (article, ogp_image) in articles.iter().zip(ogp_images) {
        let article_dir = output_root.join("articles").join(&article.slug);
        create_dir_all(&article_dir)?;
        let output = ArticleOutput::from_article(article, ogp_image.clone());
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
