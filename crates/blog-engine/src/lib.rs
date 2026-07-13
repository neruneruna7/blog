//! なぜ実装するのか:
//! - Typstブログ向けメタデータ生成の公開境界をまとめるため。
//!
//! 期待する動作:
//! - TOML入力を検証し、生成JSONと記事本文HTML fragmentへ変換する機能を提供する。
//!
//! 使い方の概要:
//! - CLIやテストから `build` または `load_normalized_articles` を呼び出す。

pub mod article;
pub mod config;
pub mod error;
pub mod html_postprocess;
pub mod new_article;
pub mod ogp;
pub mod output;
pub mod typst_compile;

use std::path::Path;

use article::{normalize_article_sources, normalize_articles, Article};
use config::{load_article_config_files, load_article_configs, load_site_config};
use error::BlogEngineError;
use output::{write_article_outputs, write_site_output};
use typst_compile::{write_article_body_html, CliTypstRunner, TypstRunner};

/// なぜ実装するのか:
/// - CLIとテストの両方から同じビルド境界を呼び出せるようにするため。
///
/// 期待する動作:
/// - 入力ルートからサイト設定、記事設定、記事本文を読み、生成物を出力ルートへ書き込む。
///
/// 使い方の概要:
/// - `build("typst-src", "generated")` のように呼び出す。
pub fn build(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
) -> Result<(), BlogEngineError> {
    let source_root = source_root.as_ref();
    let runner = CliTypstRunner::new(source_root);
    let postprocess_context = html_postprocess::PostprocessContext::production();

    build_with_typst_runner_and_postprocess_context(
        source_root,
        output_root,
        &runner,
        &postprocess_context,
    )
}
/// なぜ実装するのか:
/// - 本番ではTypst CLIを使い、テストでは外部バイナリに依存しないrunnerを注入するため。
///
/// 期待する動作:
/// - 入力ルートからサイト設定、記事設定、記事本文を読み、JSONとbody.htmlを出力する。
///
/// 使い方の概要:
/// - 通常は `build` を使い、テストや特殊な実行境界だけrunnerを明示する。
pub fn build_with_typst_runner(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    runner: &impl TypstRunner,
) -> Result<(), BlogEngineError> {
    let postprocess_context = html_postprocess::PostprocessContext::production();
    build_with_typst_runner_and_postprocess_context(
        source_root,
        output_root,
        runner,
        &postprocess_context,
    )
}

/// なぜ実装するのか:
/// - Typst実行とHTML後処理の外部境界を個別にテスト可能にするため。
///
/// 期待する動作:
/// - Typst変換後の本文fragmentを、指定された後処理文脈で変換してから検証・保存する。
///
/// 使い方の概要:
/// - 通常は `build_with_typst_runner` を使い、後処理を差し替えるテストだけで使う。
pub fn build_with_typst_runner_and_postprocess_context(
    source_root: impl AsRef<Path>,
    output_root: impl AsRef<Path>,
    runner: &impl TypstRunner,
    postprocess_context: &html_postprocess::PostprocessContext,
) -> Result<(), BlogEngineError> {
    let source_root = source_root.as_ref();
    let output_root = output_root.as_ref();

    let site = load_site_config(source_root)?;
    let article_sources = load_normalized_article_sources(source_root)?;
    let articles = article_sources
        .iter()
        .map(|source| source.article.clone())
        .collect::<Vec<Article>>();

    write_site_output(output_root, &site)?;
    write_article_outputs(output_root, &articles)?;
    for article in &article_sources {
        write_article_body_html(output_root, article, runner, postprocess_context)?;
    }

    Ok(())
}

/// なぜ実装するのか:
/// - 記事メタデータの正規化結果を、出力を伴わずにテストできるようにするため。
///
/// 期待する動作:
/// - ArticleConfig.toml群を読み、日時型を保持する検証済み `Article` を公開順で返す。
///
/// 使い方の概要:
/// - テストや将来の別出力処理から、記事一覧だけが必要な場合に呼び出す。
pub fn load_normalized_articles(
    source_root: impl AsRef<Path>,
) -> Result<Vec<Article>, BlogEngineError> {
    let article_configs = load_article_configs(source_root.as_ref())?;
    normalize_articles(article_configs)
}

fn load_normalized_article_sources(
    source_root: impl AsRef<Path>,
) -> Result<Vec<article::ArticleSource>, BlogEngineError> {
    let article_files = load_article_config_files(source_root.as_ref())?;
    normalize_article_sources(article_files)
}
