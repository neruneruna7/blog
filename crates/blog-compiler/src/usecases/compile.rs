use std::path::Path;

use crate::features::{
    article_parse::ArticleParser, article_transform::ArticleTransformer,
    artifact::ArtifactAssembler, load_source::loader::SourceLoader, ogp_generate::OgpGenerator,
};
use tracing::info;

pub mod error;

use error::Error;

#[tracing::instrument(
    skip_all,
    fields(
        typst_project_root = %typst_project_root.display(),
        article_entry_point = %article_entry_point.display(),
        ogp_main_file = %ogp_main_file.display(),
        artifact_root = %artifact_root.display()
    ),
    err
)]
pub(crate) fn article_compile(
    typst_project_root: &Path,
    article_entry_point: &Path,
    ogp_main_file: &Path,
    artifact_root: &Path,
) -> Result<usize, Error> {
    info!(
        typst_project_root = %typst_project_root.display(),
        article_entry_point = %article_entry_point.display(),
        ogp_main_file = %ogp_main_file.display(),
        artifact_root = %artifact_root.display(),
        "記事コンパイルを開始"
    );

    let input_bundle = SourceLoader::load(typst_project_root, article_entry_point)?;
    info!(
        article_count = input_bundle.articles.len(),
        "入力を読み込み完了"
    );

    let parsed_site = ArticleParser::parse(input_bundle)?;
    info!(
        article_count = parsed_site.articles.len(),
        "記事とサイト設定を解析完了"
    );

    let ogp = OgpGenerator::generate_all(&parsed_site, typst_project_root, ogp_main_file)?;
    info!(image_count = ogp.len(), "OGP画像を生成完了");

    let transformed = ArticleTransformer::transform_all(parsed_site.articles, ogp)?;
    let article_count = transformed.len();
    info!(article_count, "記事HTMLを変換完了");

    let artifact = ArtifactAssembler::assemble(transformed, &parsed_site.site)?;

    info!(output_root = %artifact_root.display(), "Artifactの書き込みを開始");
    artifact.write(artifact_root)?;
    info!(article_count, "Artifactの書き込み完了");

    Ok(article_count)
}
