use crate::{error::ArticleCompileError, usecases::compile::article_compile};

mod compiler;
pub mod domain;
mod error;
pub mod input;
pub mod output;

mod usecases;

fn main() -> Result<(), ArticleCompileError> {
    let articles = article_compile()?;
    println!("Typstコンパイル完了: {} 件", articles.len());
    Ok(())
}
