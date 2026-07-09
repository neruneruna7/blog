use std::path::Path;
use std::process::Command;

use crate::error::{BlogError, Result};

/// Typstコンパイルの最小インターフェイスである。
///
/// # なぜ実装するのか
/// 本番ではTypst CLIを呼び，テストではfakeに差し替えて外部バイナリへ依存しないためである。
///
/// # 期待する動作
/// main.typをHTMLへ，ogp.typをPNGへコンパイルする。
///
/// # 使い方の概要
/// `build::build_site`へtrait objectまたは具象型を渡す。
pub trait TypstCompiler {
    fn compile_html(&self, root: &Path, input: &Path, output: &Path) -> Result<()>;
    fn compile_png(&self, root: &Path, input: &Path, output: &Path) -> Result<()>;
}

/// Typst CLIを呼び出す実装である。
///
/// # なぜ実装するのか
/// プロトタイプではcompiler crateを埋め込まず，外部CLIを正本として使うためである。
///
/// # 期待する動作
/// `typst compile`を実行し，失敗時にはstderr要約を含むエラーを返す。
///
/// # 使い方の概要
/// CLIの`build`コマンドから`TypstCommand`を渡す。
#[derive(Clone, Copy, Debug, Default)]
pub struct TypstCommand;

impl TypstCompiler for TypstCommand {
    fn compile_html(&self, root: &Path, input: &Path, output: &Path) -> Result<()> {
        run_typst(root, input, output, &["compile", "--features", "html"])
    }

    fn compile_png(&self, root: &Path, input: &Path, output: &Path) -> Result<()> {
        run_typst(root, input, output, &["compile"])
    }
}

fn run_typst(root: &Path, input: &Path, output: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("typst")
        .args(args)
        .arg("--root")
        .arg(root)
        .arg(input)
        .arg(output)
        .output()
        .map_err(|source| BlogError::TypstFailed {
            article_dir: "<unknown>".to_owned(),
            entry: input.to_path_buf(),
            kind: "command",
            stderr: source.to_string(),
        })?;

    if status.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&status.stderr);
        Err(BlogError::TypstFailed {
            article_dir: "<unknown>".to_owned(),
            entry: input.to_path_buf(),
            kind: "command",
            stderr: summarize_stderr(&stderr),
        })
    }
}

fn summarize_stderr(stderr: &str) -> String {
    let summary = stderr.lines().take(8).collect::<Vec<_>>().join(" ");
    if summary.is_empty() {
        "<empty stderr>".to_owned()
    } else {
        summary
    }
}
