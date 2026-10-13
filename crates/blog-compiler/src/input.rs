//! ファイルシステムなどから読み込んだデータを表す入力モデル。

pub mod article;
pub mod error;
pub mod loader;
pub mod site;

/// Typst 記事とサイト設定を読み込むルート。
pub const TYP_COMPILE_ROOT: &str = "typst-src";
