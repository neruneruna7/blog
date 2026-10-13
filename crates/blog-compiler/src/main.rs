use crate::{error::ArticleCompileError, usecases::compile::article_compile};

pub mod artifact;
mod compiler;
pub mod domain;
mod error;
pub mod input;

mod usecases;

fn main() -> Result<(), ArticleCompileError> {
    let articles = article_compile()?;
    println!("Typstコンパイル完了: {} 件", articles.len());
    Ok(())
}
