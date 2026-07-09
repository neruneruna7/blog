use blog_proto::cli::{Cli, Commands};
use blog_proto::typst::TypstCommand;
use clap::Parser;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let cwd = std::env::current_dir()?;

    match cli.command {
        Commands::Build => {
            blog_proto::build::build_site(&cwd, &TypstCommand)?;
        }
        Commands::Check => blog_proto::check::check_site(&cwd)?,
        Commands::New { title } => {
            blog_proto::new_article::create_article(&cwd, &title)?;
        }
    }

    Ok(())
}
