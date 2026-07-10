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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => {
            blog_engine::build(args.source, args.output)?;
        }
    }

    Ok(())
}
