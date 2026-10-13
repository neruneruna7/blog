use std::collections::HashMap;

use crate::{
    commons::app_config::{ARTICLE_ENTRY_POINT, ARTIFACT_ROOT, OGP_MAIN_FILE, TYP_COMPILE_ROOT},
    features::{
        article_parse::ArticleParser,
        article_transform::ArticleTransformer,
        artifact::{
            ArtifactWriter,
            article::ArticleArtifact,
            site::{SiteArtifacts, SiteConfig},
        },
        input::loader::Loader,
        ogp_generate::OgpGenerator,
    },
};

pub mod error;

use error::Error;

pub(crate) fn article_compile() -> Result<usize, Error> {
    let input_bundle = Loader::load(TYP_COMPILE_ROOT, ARTICLE_ENTRY_POINT)?;
    let parsed_site = ArticleParser::parse(input_bundle)?;
    let articles = parsed_site.articles;
    let site = parsed_site.site;

    let ogp = articles
        .iter()
        .map(|(slug, article)| -> Result<_, Error> {
            Ok((
                slug.clone(),
                OgpGenerator::generate_png(article, &site, TYP_COMPILE_ROOT, OGP_MAIN_FILE)?,
            ))
        })
        .collect::<Result<HashMap<_, _>, Error>>()?;

    let transformed = ArticleTransformer::transform_all(articles, ogp)?;
    let article_count = transformed.len();
    let article_artifacts = transformed
        .into_values()
        .map(|article| ArticleArtifact::try_from_processed(article, &site))
        .collect::<Result<Vec<_>, _>>()?;
    let site_artifacts = SiteArtifacts::try_new(SiteConfig::from(&site), article_artifacts)?;

    ArtifactWriter::write(&site_artifacts, ARTIFACT_ROOT)?;

    Ok(article_count)
}
