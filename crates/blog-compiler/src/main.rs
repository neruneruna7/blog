use crate::usecases::compile::{article_compile, error::Error};

pub mod commons;
mod compiler;
pub mod features;

mod usecases;

fn main() -> Result<(), Error> {
    let article_count = article_compile()?;
    println!("記事変換・成果物書き込み完了: {article_count} 件");
    Ok(())
}
