use crate::usecases::compile::{article_compile, error::Error};

pub mod article_transform;

pub mod artifact;
pub mod commons;
mod compiler;
pub mod input;
pub mod parse_article;

mod usecases;

fn main() -> Result<(), Error> {
    let articles = article_compile()?;
    println!("記事変換完了: {} 件", articles.len());
    Ok(())
}
