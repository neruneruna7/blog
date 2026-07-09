use std::path::PathBuf;

/// blog-proto全体のエラー型である。
///
/// # なぜ実装するのか
/// 入力検証，I/O，Typst実行失敗をCLIとテストで同じ粒度で扱うためである。
///
/// # 期待する動作
/// ドメイン規則違反と外部コマンド失敗を判別できるメッセージとして返す。
///
/// # 使い方の概要
/// 公開APIは`Result<T>`を返し，呼び出し側は表示またはassertionに利用する。
#[derive(Debug, thiserror::Error)]
pub enum BlogError {
    #[error("I/O error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("TOML parse error at {path}: {source}")]
    Toml {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("missing required input: {0}")]
    MissingInput(PathBuf),
    #[error("invalid slug `{slug}` in {path}")]
    InvalidSlug { slug: String, path: PathBuf },
    #[error("duplicate article id `{id}` in {first} and {second}")]
    DuplicateId {
        id: uuid::Uuid,
        first: String,
        second: String,
    },
    #[error("duplicate article slug `{slug}` in {first} and {second}")]
    DuplicateSlug {
        slug: String,
        first: String,
        second: String,
    },
    #[error("article already exists: {0}")]
    ArticleAlreadyExists(PathBuf),
    #[error("typst {kind} failed for article `{article_dir}` at {entry}: {stderr}")]
    TypstFailed {
        article_dir: String,
        entry: PathBuf,
        kind: &'static str,
        stderr: String,
    },
}

/// blog-protoの標準Result型である。
///
/// # なぜ実装するのか
/// すべての公開APIで同一のエラー型を返し，テストとCLIの扱いを単純化するためである。
///
/// # 期待する動作
/// 成功値または`BlogError`を返す。
///
/// # 使い方の概要
/// `Result<T>`を公開関数の返り値として利用する。
pub type Result<T> = std::result::Result<T, BlogError>;

pub(crate) trait PathContext<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> PathContext<T> for std::io::Result<T> {
    fn at(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| BlogError::Io {
            path: path.into(),
            source,
        })
    }
}
