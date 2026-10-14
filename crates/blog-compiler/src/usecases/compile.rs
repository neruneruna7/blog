use std::path::Path;

use crate::features::{
    article_parse::ArticleParser, article_transform::ArticleTransformer,
    artifact::ArtifactAssembler, input::loader::Loader, ogp_generate::OgpGenerator,
};

pub mod error;

use error::Error;

pub(crate) fn article_compile(
    typst_project_root: &Path,
    article_entry_point: &Path,
    ogp_main_file: &Path,
    artifact_root: &Path,
) -> Result<usize, Error> {
    let input_bundle = Loader::load(typst_project_root, article_entry_point)?;
    let parsed_site = ArticleParser::parse(input_bundle)?;
    let ogp = OgpGenerator::generate_all(&parsed_site, typst_project_root, ogp_main_file)?;
    let transformed = ArticleTransformer::transform_all(parsed_site.articles, ogp)?;
    let article_count = transformed.len();
    let artifact = ArtifactAssembler::assemble(transformed, &parsed_site.site)?;

    artifact.write(artifact_root)?;

    Ok(article_count)
}
