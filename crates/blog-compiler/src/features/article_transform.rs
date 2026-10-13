//! ドメインパース済みの記事を中間生成物へ変換するサブシステム。

mod article;
pub mod error;
mod output;
mod postprocess;
pub mod transformer;

pub use output::{ArticleCompileWarning, ProcessedArticle};
pub use transformer::ArticleTransformer;
