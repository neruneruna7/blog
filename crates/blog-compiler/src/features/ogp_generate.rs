//! 記事とサイトのデータからOGP画像を生成するサブシステム。

pub mod error;
pub mod generator;

pub use generator::OgpGenerator;
