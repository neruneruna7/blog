use std::path::{Path, PathBuf};

/// プロジェクト内の標準パスをまとめる値である。
///
/// # なぜ実装するのか
/// `typst-src`，`site-src`，`public`，`.blog-cache`への参照を一箇所に集約するためである。
///
/// # 期待する動作
/// ワークスペースrootから各入出力パスを安定して生成する。
///
/// # 使い方の概要
/// `ProjectPaths::new(root)`で作り，build，check，newで利用する。
#[derive(Clone, Debug)]
pub struct ProjectPaths {
    pub root: PathBuf,
    pub typst_src: PathBuf,
    pub site_src: PathBuf,
    pub public: PathBuf,
    pub cache: PathBuf,
}

impl ProjectPaths {
    /// 標準ディレクトリ構造に基づくパス集合を生成する。
    ///
    /// # なぜ実装するのか
    /// 呼び出し側が文字列連結でパスを組み立てる状態を避けるためである。
    ///
    /// # 期待する動作
    /// 指定root直下の固定ディレクトリを指す値を返す。
    ///
    /// # 使い方の概要
    /// CLIのcurrent_dirまたはテストの一時ディレクトリを渡す。
    pub fn new(root: impl AsRef<Path>) -> Self {
        let root = root.as_ref().to_path_buf();
        Self {
            typst_src: root.join("typst-src"),
            site_src: root.join("site-src"),
            public: root.join("public"),
            cache: root.join(".blog-cache"),
            root,
        }
    }

    /// 記事ディレクトリ一覧の親パスを返す。
    ///
    /// # なぜ実装するのか
    /// 記事探索の入口を固定し，呼び出し側のパス知識を減らすためである。
    ///
    /// # 期待する動作
    /// `typst-src/articles`を返す。
    ///
    /// # 使い方の概要
    /// manifest探索とnewコマンドで利用する。
    pub fn articles_dir(&self) -> PathBuf {
        self.typst_src.join("articles")
    }
}
