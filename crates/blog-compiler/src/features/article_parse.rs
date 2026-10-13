//! 入力モデルをドメイン制約を満たす型へ変換する。

pub mod article;
pub mod error;
pub mod parser;
pub mod site;

pub use parser::{ArticleParser, ParsedSiteBundle};
