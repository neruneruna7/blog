use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "blog-engine",
    version,
    about = "Typstブログコンテンツを生成するCLI",
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Typstソースからブログ用の中間生成物を構築する。
    Build(BuildArgs),
    /// 新しいTypst記事の雛形を作成する。
    New(NewArgs),
}

#[derive(Debug, Args)]
struct BuildArgs {
    /// Typstプロジェクトルート兼コンテンツ入力ルート。
    #[arg(long, default_value = "typst-src", value_name = "DIR")]
    source: PathBuf,

    /// JSONおよびHTMLの出力ルート。
    #[arg(long, default_value = "generated", value_name = "DIR")]
    output: PathBuf,
}

#[derive(Debug, Args)]
struct NewArgs {
    /// 記事タイトル。省略時は標準入力から一行読む。
    #[arg(long)]
    title: Option<String>,

    /// 記事概要。省略時は正規化済みタイトルを使う。
    #[arg(long)]
    description: Option<String>,

    /// Typstプロジェクトルート兼コンテンツ入力ルート。
    #[arg(long, default_value = "typst-src", value_name = "DIR")]
    source: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => {
            blog_engine::build(args.source, args.output)?;
        }
        Commands::New(args) => {
            let title = match args.title {
                Some(title) => title,
                None => read_title_from_stdin()?,
            };
            if title.trim().is_empty() {
                println!("Error: タイトルが空文字列");
                return Ok(());
            }
            let article = blog_engine::new_article::create_article(
                args.source,
                blog_engine::new_article::NewArticleRequest {
                    title,
                    description: args.description,
                },
            )?;
            println!(
                "Created article:\\n  id:   {}\\n  slug: {}\\n  path: {}",
                article.id,
                article.slug,
                article.article_dir.display()
            );
        }
    }

    Ok(())
}

fn read_title_from_stdin() -> Result<String> {
    let mut title = String::new();
    print!("記事タイトルを入力 :>");
    io::stdout().flush()?;
    io::stdin().lock().read_line(&mut title)?;
    Ok(title)
}
