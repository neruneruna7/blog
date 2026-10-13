use crate::usecases::compile::{article_compile, error::Error};

pub mod commons;
mod compiler;
pub mod features;

mod usecases;

fn main() -> Result<(), Error> {
    let articles = article_compile()?;
    println!("記事変換完了: {} 件", articles.len());
    Ok(())
}
