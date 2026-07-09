//! なぜ実装するのか:
//! - 正規化済みメタデータのJSON書き出しを集約するため。
//!
//! 期待する動作:
//! - サイト、記事一覧、記事別メタデータをpretty JSONで生成する。
//!
//! 使い方の概要:
//! - ビルド処理の後段で `write_site_output` と `write_article_outputs` を呼び出す。

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::article::ArticleMeta;
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
///
/// 使い方の概要:
/// - 正規化済み記事メタデータの一覧を渡して呼び出す。
pub fn write_article_outputs(
    output_root: impl AsRef<Path>,
    articles: &[ArticleMeta],
) -> Result<(), BlogEngineError> {
    let output_root = output_root.as_ref();
    create_dir_all(output_root)?;
    write_json(output_root.join("article-index.json"), articles)?;

    for article in articles {
        let article_dir = output_root.join("articles").join(&article.slug);
        create_dir_all(&article_dir)?;
        write_json(article_dir.join("meta.json"), article)?;
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
