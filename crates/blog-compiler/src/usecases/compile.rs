use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    commons::app_config::{ARTICLE_ENTRY_POINT, ARTIFACT_ROOT, OGP_MAIN_FILE, TYP_COMPILE_ROOT},
    features::{
        article_transform::ArticleTransformer, article_write::ArticleWriter,
        blog_cache_loader::BlogCacheLoader, build_cache::BuildCache,
        build_cache_writer::BuildCacheWriter, load_source::loader::SourceLoader,
        metadata_write::MetadataWriter, ogp_generate::OgpGenerator, ogp_write::OgpWriter,
        source_parse::SourceParser,
    },
};
use tracing::info;

pub mod error;

use error::Error;

/// 記事とサイト設定を読み込み、HTML・OGP・メタデータを独立して更新する。
///
/// HTMLとOGPは個別にキャッシュ判定し、各成果物の書き込み成功後にBuildCacheWriterが指紋を保存する。
///
/// # Examples
///
/// ```sh
/// cargo make build-content
/// ```
///
/// # Errors
///
/// 入力の読み込み・解析、HTML・OGPの生成、各成果物の書き込み、またはキャッシュ更新に失敗した場合に返す。
#[tracing::instrument(err)]
pub(crate) fn article_compile() -> Result<usize, Error> {
    let typst_project_root = Path::new(TYP_COMPILE_ROOT);
    let article_entry_point = Path::new(ARTICLE_ENTRY_POINT);
    let ogp_main_file = Path::new(OGP_MAIN_FILE);
    let artifact_root = Path::new(ARTIFACT_ROOT);

    info!("記事コンパイルを開始");

    let source = SourceLoader::load(typst_project_root, article_entry_point)?;
    info!(article_count = source.articles.len(), "入力を読み込み完了");

    let parsed_source = SourceParser::parse(source)?;
    info!(
        article_count = parsed_source.articles.len(),
        "記事とサイト設定を解析完了"
    );

    let cache_input = BlogCacheLoader::load(&parsed_source, typst_project_root, artifact_root)?;
    let build_plan = BuildCache::new(&cache_input);
    let mut build_cache_writer = BuildCacheWriter::new(cache_input, artifact_root);
    let total_count = parsed_source.articles.len();
    let html_cache_hit_count = total_count - build_plan.html_count();
    let ogp_cache_hit_count = total_count - build_plan.ogp_count();

    let html_articles = parsed_source
        .articles
        .iter()
        .filter(|(slug, _)| build_plan.needs_html(slug))
        .map(|(slug, article)| (slug.clone(), article.clone()))
        .collect::<HashMap<_, _>>();
    let compiled_html = ArticleTransformer::transform_all(html_articles)?;
    let mut html_written_count = 0;
    for (slug, html) in compiled_html {
        ArticleWriter::write(&slug, &html, artifact_root)?;
        html_written_count += 1;
        build_cache_writer.record_html_success(&slug)?;
    }
    info!(
        cache_hit_count = html_cache_hit_count,
        total_count,
        written_count = html_written_count,
        "記事HTMLのキャッシュ・書き込み結果"
    );

    let mut ogp_written_count = 0;
    for (slug, article) in &parsed_source.articles {
        if !build_plan.needs_ogp(slug) {
            continue;
        }
        let png = OgpGenerator::generate_png(
            article,
            &parsed_source.site,
            typst_project_root,
            ogp_main_file,
        )?;
        OgpWriter::write(slug, &png, artifact_root)?;
        ogp_written_count += 1;
        build_cache_writer.record_ogp_success(slug)?;
    }
    info!(
        cache_hit_count = ogp_cache_hit_count,
        total_count,
        written_count = ogp_written_count,
        "OGP画像のキャッシュ・書き込み結果"
    );

    MetadataWriter::write(&parsed_source, artifact_root)?;
    info!("メタデータの書き込み完了");

    let current_slugs = parsed_source
        .articles
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    ArticleWriter::remove_obsolete(&current_slugs, artifact_root)?;
    MetadataWriter::remove_obsolete(&parsed_source, artifact_root)?;
    OgpWriter::remove_obsolete(&current_slugs, artifact_root)?;
    build_cache_writer.remove_obsolete(&current_slugs)?;
    info!(article_count = total_count, "コンパイル成果物の更新完了");

    Ok(total_count)
}
