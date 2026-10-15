//! 入力モデルをドメイン制約を満たす型へ変換する。

pub mod article;
pub mod error;
pub mod parser;
pub mod site;
pub mod source;

pub use parser::ArticleParser;
pub use source::ParsedSource;
