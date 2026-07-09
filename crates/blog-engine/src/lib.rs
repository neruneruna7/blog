//! なぜ実装するのか:
//! - Typstブログ向けメタデータ生成の公開境界をまとめるため。
//!
//! 期待する動作:
//! - TOML入力を検証し、生成JSON契約へ変換する機能を提供する。
//!
//! 使い方の概要:
//! - CLIやテストから `build` または `load_normalized_articles` を呼び出す。

pub mod article;
pub mod config;
pub mod error;
pub mod output;

use std::path::Path;

use article::{normalize_articles, ArticleMeta};
use config::{load_article_configs, load_site_config};
use error::BlogEngineError;
use output::{write_article_outputs, write_site_output};

/// なぜ実装するのか:
/// - CLIとテストの両方から同じビルド境界を呼び出せるようにするため。
///
/// 期待する動作:
/// - 入力ルートからサイト設定と記事設定を読み、生成JSONを出力ルートへ書き込む。
///
/// 使い方の概要:
/// - `build("typst-src", "generated")` のように呼び出す。
pub fn build(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
) -> Result<(), BlogEngineError> {
    let source_root = source_root.as_ref();
    let output_root = output_root.as_ref();

    let site = load_site_config(source_root)?;
    let articles = load_normalized_articles(source_root)?;

    write_site_output(output_root, &site)?;
    write_article_outputs(output_root, &articles)?;

    Ok(())
}

/// なぜ実装するのか:
/// - 記事メタデータの正規化結果を、出力を伴わずにテストできるようにするため。
///
/// 期待する動作:
/// - ArticleConfig.toml群を読み、検証済みの記事メタデータを公開日時降順で返す。
///
/// 使い方の概要:
/// - テストや将来の別出力処理から、記事一覧だけが必要な場合に呼び出す。
pub fn load_normalized_articles(
    source_root: impl AsRef<Path>,
) -> Result<Vec<ArticleMeta>, BlogEngineError> {
    let article_configs = load_article_configs(source_root.as_ref())?;
    normalize_articles(article_configs)
}
