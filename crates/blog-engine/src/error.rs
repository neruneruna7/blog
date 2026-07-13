//! なぜ実装するのか:
//! - メタデータ生成で発生する失敗を型で分類するため。
//!
//! 期待する動作:
//! - I/O、TOML、検証、走査、Typst OGP描画の失敗を呼び出し側へ返す。
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
    /// - OSの乱数源を使えない失敗を記事作成の入力検証失敗と区別するため。
    ///
    /// 期待する動作:
    /// - 乱数取得に失敗した理由を保持して呼び出し側へ返す。
    ///
    /// 使い方の概要:
    /// - 新規記事のslug生成時にOS乱数源が失敗した場合に返す。
    #[error("failed to obtain random bytes: {source}")]
    RandomSource {
        #[source]
        source: getrandom::Error,
    },

    /// なぜ実装するのか:
    /// - 新規記事の確定先が既存記事を上書きしないことを明示するため。
    ///
    /// 期待する動作:
    /// - 競合した確定先パスを含めて作成を拒否する。
    ///
    /// 使い方の概要:
    /// - stagingディレクトリを確定先へ移動する直前の存在確認で返す。
    #[error("article path already exists: {path}")]
    ArticlePathExists { path: PathBuf },

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
    /// - OGP用Typstプロセスを開始できない失敗を、コンパイル失敗と区別するため。
    ///
    /// 期待する動作:
    /// - テンプレート、出力先、OS由来の開始失敗を保持して報告する。
    ///
    /// 使い方の概要:
    /// - `TypstOgpRenderer` の `Command::output` が失敗した場合に返す。
    #[error("failed to start Typst OGP process: template={template_path}, output={output_path}, source={source}")]
    OgpTypstProcessStart {
        template_path: PathBuf,
        output_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// なぜ実装するのか:
    /// - OGP用Typstが非ゼロ終了した失敗を、プロセス開始失敗と区別するため。
    ///
    /// 期待する動作:
    /// - 終了コード、stderr、テンプレート、出力先を保持して報告する。
    ///
    /// 使い方の概要:
    /// - OGP用Typstプロセスがコンパイルに失敗した場合に返す。
    #[error("Typst OGP compilation failed: template={template_path}, output={output_path}, exit_code={status_code:?}, stderr={stderr}")]
    OgpTypstCompile {
        template_path: PathBuf,
        output_path: PathBuf,
        status_code: Option<i32>,
        stderr: String,
    },

    /// なぜ実装するのか:
    /// - Typstが成功終了したにもかかわらずPNGを生成しなかった状態を検出するため。
    ///
    /// 期待する動作:
    /// - テンプレート、出力先、期待寸法を含む失敗を返す。
    ///
    /// 使い方の概要:
    /// - Typst終了後の出力ファイル存在確認で返す。
    #[error("OGP PNG was not generated: template={template_path}, output={output_path}, expected_dimensions={expected_width}x{expected_height}")]
    OgpOutputMissing {
        template_path: PathBuf,
        output_path: PathBuf,
        expected_width: u32,
        expected_height: u32,
    },

    /// なぜ実装するのか:
    /// - OGPテンプレートの読み込み失敗をTypstプロセス失敗と区別するため。
    ///
    /// 期待する動作:
    /// - テンプレート、出力先、OS由来の読み込み失敗を保持して報告する。
    ///
    /// 使い方の概要:
    /// - OGPハッシュとコンパイル引数を準備する前のテンプレート読み込みで返す。
    #[error("failed to read OGP template: template={template_path}, output={output_path}, source={source}")]
    OgpTemplateRead {
        template_path: PathBuf,
        output_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// なぜ実装するのか:
    /// - 生成PNGを検証するためのファイル読み込み失敗を区別するため。
    ///
    /// 期待する動作:
    /// - テンプレート、出力先、OS由来の読み込み失敗を保持して報告する。
    ///
    /// 使い方の概要:
    /// - Typst終了後にPNGバイト列を読み込めない場合に返す。
    #[error("failed to read generated OGP PNG: template={template_path}, output={output_path}, source={source}")]
    OgpOutputRead {
        template_path: PathBuf,
        output_path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// なぜ実装するのか:
    /// - PNGシグネチャまたはIHDRを読めない出力を、寸法不一致と区別するため。
    ///
    /// 期待する動作:
    /// - テンプレート、出力先、期待寸法、不正理由を保持して拒否する。
    ///
    /// 使い方の概要:
    /// - `validate_ogp_png` がPNGヘッダの不変条件に違反した場合に返す。
    #[error("invalid OGP PNG header: template={template_path}, output={output_path}, expected_dimensions={expected_width}x{expected_height}, reason={message}")]
    OgpPngInvalid {
        template_path: PathBuf,
        output_path: PathBuf,
        expected_width: u32,
        expected_height: u32,
        message: String,
    },

    /// なぜ実装するのか:
    /// - 有効なPNGでもOGP契約の寸法でない出力を拒否するため。
    ///
    /// 期待する動作:
    /// - 期待寸法と実寸法、テンプレート、出力先を同時に報告する。
    ///
    /// 使い方の概要:
    /// - IHDRから寸法を読み取った直後の契約検証で返す。
    #[error("OGP PNG dimensions mismatch: template={template_path}, output={output_path}, expected={expected_width}x{expected_height}, actual={actual_width}x{actual_height}")]
    OgpPngDimensions {
        template_path: PathBuf,
        output_path: PathBuf,
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
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
    /// - OS乱数源の失敗を公開エラー型へ安全に変換するため。
    ///
    /// 期待する動作:
    /// - 乱数APIが返したエラー原因を失わずに保持する。
    ///
    /// 使い方の概要:
    /// - slug生成処理から呼び出す。
    pub(crate) fn random_source_error(source: getrandom::Error) -> Self {
        Self::RandomSource { source }
    }
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
