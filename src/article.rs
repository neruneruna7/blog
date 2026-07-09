use std::path::PathBuf;

use crate::config::ArticleConfig;

/// 検出済み記事を表すmanifest項目である。
///
/// # なぜ実装するのか
/// 編集用ディレクトリ名と公開slugを分離し，公開URLがローカル名へ依存しないようにするためである。
///
/// # 期待する動作
/// 入力パス，記事設定，公開対象かどうかを一つの値として保持する。
///
/// # 使い方の概要
/// `manifest::load_manifest`で生成し，build，check，renderへ渡す。
#[derive(Clone, Debug)]
pub struct ArticleManifest {
    pub dir_name: String,
    pub dir_path: PathBuf,
    pub config_path: PathBuf,
    pub config: ArticleConfig,
}

impl ArticleManifest {
    /// 記事が公開対象であるかを返す。
    ///
    /// # なぜ実装するのか
    /// draft除外の条件を呼び出し側で重複させないためである。
    ///
    /// # 期待する動作
    /// `draft = false`ならtrue，`draft = true`ならfalseを返す。
    ///
    /// # 使い方の概要
    /// index，feed，sitemap，public出力の対象選別に使う。
    pub fn is_public(&self) -> bool {
        !self.config.draft
    }
}
