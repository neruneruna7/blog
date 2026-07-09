//! Typstで書いた記事を静的サイトへ変換するプロトタイプである。
//!
//! # なぜ実装するのか
//! CLIだけに処理を閉じると，設定検証やビルド処理を振る舞いからテストしにくくなるためである。
//!
//! # 期待する動作
//! 設定読込，manifest生成，Typst実行，HTML描画，新規記事作成を小さな公開APIとして提供する。
//!
//! # 使い方の概要
//! `build::build_site`，`check::check_site`，`new_article::create_article`をCLIまたはテストから呼び出す。

pub mod article;
pub mod build;
pub mod check;
pub mod cli;
pub mod config;
pub mod error;
pub mod manifest;
pub mod new_article;
pub mod paths;
pub mod render;
pub mod typst;

pub use error::{BlogError, Result};
