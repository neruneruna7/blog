//! ドメインパース済みの記事を中間生成物へ変換するサブシステム。

mod article;
pub mod error;
mod postprocess;
pub mod transformer;

pub use article::{ArticleCompileWarning, TransformedArticle};
pub use transformer::ArticleTransformer;
