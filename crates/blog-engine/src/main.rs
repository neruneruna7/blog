use std::path::PathBuf;

use anyhow::{bail, Context, Result};

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "build".to_string());

    if command != "build" {
        bail!("unsupported command: {command}");
    }

    let mut source = PathBuf::from("typst-src");
    let mut output = PathBuf::from("generated");

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--source" => {
                source = args
                    .next()
                    .map(PathBuf::from)
                    .context("--source requires a path")?;
            }
            "--output" => {
                output = args
                    .next()
                    .map(PathBuf::from)
                    .context("--output requires a path")?;
            }
            unknown => bail!("unsupported argument: {unknown}"),
        }
    }

    blog_engine::build(source, output)?;

    Ok(())
}
