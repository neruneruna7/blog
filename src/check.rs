use std::path::Path;

use crate::error::Result;
use crate::manifest::load_manifest;

/// 入力だけを検証する。
///
/// # なぜ実装するのか
/// CIや編集時に，公開成果物を書かずに設定エラーだけを検出するためである。
///
/// # 期待する動作
/// manifest検証を実行し，`public/`やcacheを作成しない。
///
/// # 使い方の概要
/// CLIの`check`からrootを渡して呼び出す。
pub fn check_site(root: impl AsRef<Path>) -> Result<()> {
    load_manifest(root).map(|_| ())
}
