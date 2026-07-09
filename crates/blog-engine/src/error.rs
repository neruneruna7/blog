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
}
