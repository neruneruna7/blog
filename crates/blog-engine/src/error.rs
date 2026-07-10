//! なぜ実装するのか:
//! - メタデータ生成で発生する失敗を型で分類するため。
//!
//! 期待する動作:
//! - I/O、TOML、検証、走査の失敗を呼び出し側へ返す。
//!
//! 使い方の概要:
//! - ライブラリ公開関数の `Result` エラー型として利用する。

use std::path::PathBuf;

use thiserror::Error;

/// なぜ実装するのか:
/// - メタデータ生成の失敗原因を、I/O、TOML構文、ドメイン検証に分けて扱うため。
///
/// 期待する動作:
/// - 呼び出し側が失敗の種類と対象ファイルを把握できるエラーを返す。
///
/// 使い方の概要:
/// - ライブラリ内の公開関数はこの型を `Result` のエラーとして返す。
#[derive(Debug, Error)]
pub enum BlogEngineError {
    /// なぜ実装するのか:
    /// - ファイル読み書きに失敗したパスを保持するため。
    ///
    /// 期待する動作:
    /// - OS由来のI/Oエラーと対象パスを同時に報告する。
    ///
    /// 使い方の概要:
    /// - `std::fs` 操作の失敗を変換して返す。
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// なぜ実装するのか:
    /// - TOMLとして読めない入力ファイルを区別するため。
    ///
    /// 期待する動作:
    /// - TOMLデコードエラーと対象パスを同時に報告する。
    ///
    /// 使い方の概要:
    /// - `toml::from_str` の失敗を変換して返す。
    #[error("TOML parse error at {path}: {source}")]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },

    /// なぜ実装するのか:
    /// - 入力値がドメイン規則に反したことを明示するため。
    ///
    /// 期待する動作:
    /// - フィールド名や一意性違反などの検証内容をメッセージとして報告する。
    ///
    /// 使い方の概要:
    /// - 検証関数から `Validation { message }` として返す。
    #[error("validation error: {message}")]
    Validation { message: String },

    /// なぜ実装するのか:
    /// - 記事本文の `main.typ` が存在しない入力違反を区別するため。
    ///
    /// 期待する動作:
    /// - slugと期待した入力パスを含めて報告する。
    ///
    /// 使い方の概要:
    /// - Typst本文生成の前提チェックで返す。
    #[error("missing main.typ for article {slug} at {input_path}")]
    MissingMainTyp { slug: String, input_path: PathBuf },

    /// なぜ実装するのか:
    /// - Typst CLIの失敗をI/Oや本文検証失敗と区別するため。
    ///
    /// 期待する動作:
    /// - slug、入力パス、出力パス、stderrを含めて報告する。
    ///
    /// 使い方の概要:
    /// - Typst subprocessが非ゼロ終了した場合に返す。
    #[error("Typst compilation failed for article {slug}: input={input_path}, output={output_path}, stderr={stderr}")]
    TypstCompile {
        slug: String,
        input_path: PathBuf,
        output_path: PathBuf,
        stderr: String,
    },

    /// なぜ実装するのか:
    /// - 生成HTMLが埋め込み可能な本文fragmentではない状態を明示するため。
    ///
    /// 期待する動作:
    /// - slugと検証対象の出力パスを含めて報告する。
    ///
    /// 使い方の概要:
    /// - HTML正規化後の空文字または禁止タグ検出で返す。
    #[error("invalid body HTML for article {slug} at {output_path}: {message}")]
    InvalidBodyHtml {
        slug: String,
        output_path: PathBuf,
        message: String,
    },

    /// なぜ実装するのか:
    /// - `body.html` の書き込み失敗を、一般I/Oではなく本文生成境界の失敗として示すため。
    ///
    /// 期待する動作:
    /// - slug、出力パス、OS由来のI/Oエラーを同時に報告する。
    ///
    /// 使い方の概要:
    /// - 正規化済み本文HTML fragmentの保存に失敗した場合に返す。
    #[error("failed to write body.html for article {slug} at {output_path}: {source}")]
    BodyHtmlWrite {
        slug: String,
        output_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// なぜ実装するのか:
    /// - ローカルHTML変換の不変条件違反を外部取得失敗と区別するため。
    ///
    /// 期待する動作:
    /// - HTML rewriterの初期化・変換失敗を明示して返す。
    ///
    /// 使い方の概要:
    /// - `html_postprocess` の内部失敗を変換して返す。
    #[error("HTML postprocess error: {message}")]
    HtmlPostprocess { message: String },

    /// なぜ実装するのか:
    /// - 記事設定探索時のディレクトリ走査失敗を扱うため。
    ///
    /// 期待する動作:
    /// - `walkdir` の失敗内容を保持して返す。
    ///
    /// 使い方の概要:
    /// - `WalkDir` のエントリ取得エラーを変換して返す。
    #[error("walkdir error: {0}")]
    WalkDir(#[from] walkdir::Error),
}

impl BlogEngineError {
    /// なぜ実装するのか:
    /// - 外部の `TypstRunner` 実装がTypst変換失敗を同じ型で返せるようにするため。
    ///
    /// 期待する動作:
    /// - slug、入力パス、出力パス、stderrを保持する `TypstCompile` エラーを作る。
    ///
    /// 使い方の概要:
    /// - カスタムrunnerやテストrunnerでTypst変換失敗を表現するときに使う。
    pub fn typst_compile_error(
        slug: impl Into<String>,
        input_path: impl Into<PathBuf>,
        output_path: impl Into<PathBuf>,
        stderr: impl Into<String>,
    ) -> Self {
        Self::typst_compile(slug, input_path, output_path, stderr)
    }

    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Io {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn toml(path: impl Into<PathBuf>, source: toml::de::Error) -> Self {
        Self::Toml {
            path: path.into(),
            source,
        }
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    pub(crate) fn missing_main_typ(
        slug: impl Into<String>,
        input_path: impl Into<PathBuf>,
    ) -> Self {
        Self::MissingMainTyp {
            slug: slug.into(),
            input_path: input_path.into(),
        }
    }

    pub(crate) fn typst_compile(
        slug: impl Into<String>,
        input_path: impl Into<PathBuf>,
        output_path: impl Into<PathBuf>,
        stderr: impl Into<String>,
    ) -> Self {
        Self::TypstCompile {
            slug: slug.into(),
            input_path: input_path.into(),
            output_path: output_path.into(),
            stderr: stderr.into(),
        }
    }

    pub(crate) fn invalid_body_html(
        slug: impl Into<String>,
        output_path: impl Into<PathBuf>,
        message: impl Into<String>,
    ) -> Self {
        Self::InvalidBodyHtml {
            slug: slug.into(),
            output_path: output_path.into(),
            message: message.into(),
        }
    }

    pub(crate) fn body_html_write(
        slug: impl Into<String>,
        output_path: impl Into<PathBuf>,
        source: std::io::Error,
    ) -> Self {
        Self::BodyHtmlWrite {
            slug: slug.into(),
            output_path: output_path.into(),
            source,
        }
    }

    pub(crate) fn html_postprocess(message: impl Into<String>) -> Self {
        Self::HtmlPostprocess {
            message: message.into(),
        }
    }
}
