use std::path::Path;

use clap::Parser;

use crate::{
    commons::app_config::{ARTICLE_ENTRY_POINT, ARTIFACT_ROOT, OGP_MAIN_FILE, TYP_COMPILE_ROOT},
    usecases::compile::{article_compile, error::Error},
};
use tracing_subscriber::EnvFilter;

pub mod commons;
mod compiler;
pub mod features;

mod usecases;

#[derive(Debug, Parser)]
#[command(
    name = "blog-compiler",
    version,
    about = "Typstの記事からWebサイト用の成果物を生成するCLI"
)]
struct Cli {}

fn main() -> Result<(), Error> {
    Cli::parse();
    init_logging();

    let article_count = article_compile(
        Path::new(TYP_COMPILE_ROOT),
        Path::new(ARTICLE_ENTRY_POINT),
        Path::new(OGP_MAIN_FILE),
        Path::new(ARTIFACT_ROOT),
    )?;
    println!("記事変換・成果物書き込み完了: {article_count} 件");
    Ok(())
}

fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("blog_compiler=info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
