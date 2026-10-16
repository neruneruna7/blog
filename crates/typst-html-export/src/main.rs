use std::{error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let html = typst_html_export::compile_html("typst-src", "articles/typst-blog/main.typ")?;

    fs::create_dir_all("output")?;
    fs::write(".temp/output/index.html", html)?;

    Ok(())
}
