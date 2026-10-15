use std::path::Path;

use clap::{Args, Parser, Subcommand};

use crate::{
    commons::app_config::{ARTICLE_ENTRY_POINT, ARTIFACT_ROOT, OGP_MAIN_FILE, TYP_COMPILE_ROOT},
    usecases::{
        compile::article_compile,
        create_new_article::{NewArticleRequest, create_new_article},
    },
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
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// 記事をコンパイルしてサイト用の成果物を生成する。
    Build,
    /// 記事テンプレートから新しい記事を作成する。
    New(NewArticleArgs),
}

#[derive(Debug, Args)]
struct NewArticleArgs {
    /// 使用するテンプレート名。省略時は名前順で先頭のテンプレートを使う。
    #[arg(long, value_name = "NAME")]
    template: Option<String>,

    /// 記事タイトル。省略時は既定値を使う。
    #[arg(long)]
    title: Option<String>,

    /// 記事概要。省略時はタイトルを使う。
    #[arg(long)]
    description: Option<String>,

    /// 記事タグ。複数指定できる。
    #[arg(long = "tag")]
    tags: Vec<String>,

    /// Typstプロジェクトルート。
    #[arg(long, default_value = TYP_COMPILE_ROOT, value_name = "DIR")]
    source: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    init_logging();

    match cli.command {
        Some(Command::New(args)) => {
            let article = create_new_article(
                Path::new(&args.source),
                NewArticleRequest {
                    template: args.template,
                    title: args.title,
                    description: args.description,
                    tags: args.tags,
                },
            )?;
            println!(
                "新規記事を作成: slug={} template={} path={}",
                article.slug,
                article.template,
                article.article_dir.display()
            );
        }
        Some(Command::Build) | None => {
            let article_count = article_compile(
                Path::new(TYP_COMPILE_ROOT),
                Path::new(ARTICLE_ENTRY_POINT),
                Path::new(OGP_MAIN_FILE),
                Path::new(ARTIFACT_ROOT),
            )?;
            println!("記事変換・成果物書き込み完了: {article_count} 件");
        }
    }

    Ok(())
}

fn init_logging() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("blog_compiler=info"));

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
