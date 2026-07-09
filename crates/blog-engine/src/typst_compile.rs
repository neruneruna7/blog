//! なぜ実装するのか:
//! - Typst本文ソースから埋め込み用HTML fragmentを生成する境界を集約するため。
//!
//! 期待する動作:
//! - `main.typ` をTypst CLIでHTMLへ変換し、本文部分だけを `body.html` として保存する。
//!
//! 使い方の概要:
//! - ビルド処理から `write_article_body_html` を記事ごとに呼び出す。

use std::fs;
use std::path::Path;
use std::process::Command;

use crate::article::ArticleSource;
use crate::error::BlogEngineError;

/// なぜ実装するのか:
/// - Typst実行境界を本番CLIとテスト用runnerで差し替えられるようにするため。
///
/// 期待する動作:
/// - 入力Typstファイルを受け取り、指定されたHTMLファイルへ変換する。
///
/// 使い方の概要:
/// - 本番では `CliTypstRunner` を使い、テストでは小さなfake runnerを実装する。
pub trait TypstRunner {
    /// なぜ実装するのか:
    /// - TypstからHTMLへの変換処理を抽象化するため。
    ///
    /// 期待する動作:
    /// - 成功時は `output_path` にHTMLファイルを作成し、失敗時は型付きエラーを返す。
    ///
    /// 使い方の概要:
    /// - `write_article_body_html` から記事slugと入出力パスを渡して呼び出す。
    fn compile(
        &self,
        slug: &str,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), BlogEngineError>;
}

/// なぜ実装するのか:
/// - リポジトリにTypst crateを追加せず、導入済みCLIを低リスクな変換境界として使うため。
///
/// 期待する動作:
/// - `typst compile --features html <input> <output>` を実行し、stderrを失敗文脈に含める。
///
/// 使い方の概要:
/// - `build` の既定runnerとして利用する。
#[derive(Debug, Clone, Copy, Default)]
pub struct CliTypstRunner;

impl TypstRunner for CliTypstRunner {
    fn compile(
        &self,
        slug: &str,
        input_path: &Path,
        output_path: &Path,
    ) -> Result<(), BlogEngineError> {
        let output = Command::new("typst")
            .arg("compile")
            .arg("--features")
            .arg("html")
            .arg(input_path)
            .arg(output_path)
            .output()
            .map_err(|source| {
                BlogEngineError::typst_compile(
                    slug,
                    input_path,
                    output_path,
                    format!("failed to start typst CLI: {source}"),
                )
            })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(BlogEngineError::typst_compile(
            slug,
            input_path,
            output_path,
            if stderr.is_empty() {
                format!("typst exited with status {}", output.status)
            } else {
                stderr
            },
        ))
    }
}

/// なぜ実装するのか:
/// - 記事単位で `main.typ` から `generated/articles/<slug>/body.html` を生成するため。
///
/// 期待する動作:
/// - `main.typ` の存在、Typst変換、本文fragment正規化、fragment検証、保存を順に行う。
///
/// 使い方の概要:
/// - 正規化済み `ArticleSource` と出力ルート、Typst runnerを渡して呼び出す。
pub(crate) fn write_article_body_html(
    output_root: impl AsRef<Path>,
    article: &ArticleSource,
    runner: &impl TypstRunner,
) -> Result<(), BlogEngineError> {
    debug_assert_eq!(article.main_typ_path, article.article_dir.join("main.typ"));

    if !article.main_typ_path.is_file() {
        return Err(BlogEngineError::missing_main_typ(
            &article.meta.slug,
            &article.main_typ_path,
        ));
    }

    let article_output_dir = output_root
        .as_ref()
        .join("articles")
        .join(&article.meta.slug);
    fs::create_dir_all(&article_output_dir)
        .map_err(|source| BlogEngineError::io(&article_output_dir, source))?;

    let full_html_path = article_output_dir.join("body.full.tmp.html");
    runner.compile(&article.meta.slug, &article.main_typ_path, &full_html_path)?;

    let full_html = fs::read_to_string(&full_html_path)
        .map_err(|source| BlogEngineError::io(&full_html_path, source))?;
    let body_html_path = article_output_dir.join("body.html");
    let fragment = normalize_body_fragment(&full_html);
    validate_body_fragment(&fragment, &article.meta.slug, &body_html_path)?;
    fs::write(&body_html_path, format!("{fragment}\n")).map_err(|source| {
        BlogEngineError::body_html_write(&article.meta.slug, &body_html_path, source)
    })?;

    let _ = fs::remove_file(full_html_path);

    Ok(())
}

/// なぜ実装するのか:
/// - 将来Astroの `set:html` に渡せる本文fragmentだけを保証するため。
///
/// 期待する動作:
/// - 空文字と、document-levelタグを含むHTMLを拒否する。
///
/// 使い方の概要:
/// - Typst出力から本文を取り出した後、保存前に呼び出す。
pub fn validate_body_fragment(
    html: &str,
    slug: &str,
    output_path: impl AsRef<Path>,
) -> Result<(), BlogEngineError> {
    let trimmed = html.trim();
    if trimmed.is_empty() {
        return Err(BlogEngineError::invalid_body_html(
            slug,
            output_path.as_ref(),
            "body HTML must not be empty",
        ));
    }

    let lowercased = trimmed.to_lowercase();
    for forbidden in ["<!doctype", "<html", "<head", "<body"] {
        if lowercased.contains(forbidden) {
            return Err(BlogEngineError::invalid_body_html(
                slug,
                output_path.as_ref(),
                format!("body HTML must not contain {forbidden}"),
            ));
        }
    }

    Ok(())
}

/// Typst CLIが完全なHTML文書を返す場合に、`<body>...</body>` の内側だけを取り出す。
fn normalize_body_fragment(html: &str) -> String {
    let lowercased = html.to_lowercase();
    let Some(body_start) = lowercased.find("<body") else {
        return html.trim().to_string();
    };
    let Some(open_end) = lowercased[body_start..].find('>') else {
        return html.trim().to_string();
    };
    let content_start = body_start + open_end + 1;
    let Some(body_end) = lowercased[content_start..].find("</body>") else {
        return html.trim().to_string();
    };

    html[content_start..content_start + body_end]
        .trim()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validation_accepts_non_empty_fragment() {
        let actual = validate_body_fragment("<h1>Hello</h1>", "hello", "body.html");

        assert!(actual.is_ok());
    }

    #[test]
    fn validation_rejects_empty_fragment() {
        let actual = validate_body_fragment(" \n\t", "hello", "body.html");

        assert!(matches!(
            actual,
            Err(BlogEngineError::InvalidBodyHtml { .. })
        ));
    }

    #[test]
    fn validation_rejects_document_level_tags() {
        for html in [
            "<!doctype html>",
            "<html lang=\"ja\"></html>",
            "<head></head>",
            "<body><p>Hello</p></body>",
        ] {
            let actual = validate_body_fragment(html, "hello", "body.html");

            assert!(
                matches!(actual, Err(BlogEngineError::InvalidBodyHtml { .. })),
                "{html:?} should be rejected"
            );
        }
    }

    #[test]
    fn normalize_extracts_body_content_from_full_html() {
        let actual = normalize_body_fragment(
            "<!DOCTYPE html><html><head></head><body><h2>Hello</h2><p>Text</p></body></html>",
        );

        assert_eq!(actual, "<h2>Hello</h2><p>Text</p>");
    }
}
