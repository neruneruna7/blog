use std::{
    collections::{HashMap, HashSet},
    error::Error as StdError,
    fs,
    path::{Path, PathBuf},
};

use crate::error::{
    ArticleCompileError, ArticleValidationError, ArticleWriteError, OgpSiteConfigError,
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use typst_html_export::{OGP_HEIGHT_PX, OGP_WIDTH_PX};

mod postprocess;

pub fn article_compile() -> Result<(), Box<dyn StdError>> {
    let article_inputs = RawArticleInput::read_articles(TYP_COMPILE_ROOT)?;
    let validated_article_inputs = RawArticleInput::validate_all(article_inputs)?;
    let postprocessors = postprocess::Pipeline::production();
    let compiled_articles: HashMap<Slug, CompiledArticle> = validated_article_inputs
        .into_iter()
        .map(|article| compile(&article, &postprocessors))
        .filter_map(Result::ok)
        .map(|article| (article.config.slug.clone(), article))
        .collect();
    let site = OgpSiteConfig::read(TYP_COMPILE_ROOT)?;
    let compiled_article_count = compiled_articles.len();
    let articles: HashMap<Slug, Article> = compiled_articles
        .into_iter()
        .map(|(slug, compiled)| {
            let input = typst_html_export::OgpInput {
                site_title: site.title.clone(),
                article_title: compiled.config.title.0.clone(),
                description: compiled.config.description.0.clone(),
                tags: compiled
                    .config
                    .tags
                    .iter()
                    .map(|tag| tag.0.clone())
                    .collect(),
            };
            let ogp =
                typst_html_export::compile_ogp_png(TYP_COMPILE_ROOT, OGP_ENTRY_POINT, &input)?;

            let metadata = ArticleOutput {
                id: compiled.config.id,
                slug: slug.0.clone(),
                title: compiled.config.title.0,
                description: compiled.config.description.0,
                published_at: compiled.config.published_at.0.to_rfc3339(),
                updated_at: compiled
                    .config
                    .updated_at
                    .map(|updated_at| updated_at.to_rfc3339()),
                tags: compiled.config.tags.into_iter().map(|tag| tag.0).collect(),
                url: format!("/articles/{}/", slug.0),
                ogp_image: OgpImageMeta {
                    path: format!("/{OGP_ARTICLES_DIRECTORY}/{}.png", slug.0),
                    width: OGP_WIDTH_PX,
                    height: OGP_HEIGHT_PX,
                    mime_type: "image/png".to_owned(),
                    alt: format!("{} | {}", input.article_title, site.title),
                },
            };
            let article = Article {
                metadata,
                ogp_png: ogp,
                body_html: compiled.postprocess.body_html,
            };

            Ok((slug, article))
        })
        .collect::<Result<HashMap<_, _>, typst_html_export::TypstHtmlExportError>>()?;
    debug_assert_eq!(articles.len(), compiled_article_count);

    articles
        .into_values()
        .try_for_each(|article| article.write(ARTICLE_OUTPUT_ROOT))?;

    Ok(())
}

// 複数ユースケースを横断する必要があるだろうが...
// まぁ，その時にどこに置くべきか見えてくるだろう．
const TYP_COMPILE_ROOT: &str = "typst-src";
const ARTICLE_OUTPUT_ROOT: &str = "generated/articles";
const PUBLIC_ROOT_DIRECTORY: &str = "public-root";
const OGP_ARTICLES_DIRECTORY: &str = "ogp/articles";
const BODY_HTML_FILE: &str = "body.html";
const ARTICLE_METADATA_FILE: &str = "meta.json";
const SITE_CONFIG_FILE: &str = "site.toml";
const ARTICLES_PATH: &str = "articles";
const ENTRY_POINT: &str = "main.typ";
const OGP_ENTRY_POINT: &str = "commons/ogp/template.typ";
const CONFIG_NAME: &str = "ArticleConfig.toml";
const MAX_TITLE_CHARACTERS: usize = 48;
const MAX_DESCRIPTION_CHARACTERS: usize = 130;

#[derive(Debug, Clone, Deserialize)]
struct OgpSiteConfig {
    title: String,
}

impl OgpSiteConfig {
    fn read(root: impl AsRef<Path>) -> Result<Self, OgpSiteConfigError> {
        let path = root.as_ref().join(SITE_CONFIG_FILE);
        let contents = fs::read_to_string(&path).map_err(|source| OgpSiteConfigError::Read {
            path: path.clone(),
            source,
        })?;
        let site: Self = toml::from_str(&contents).map_err(|source| OgpSiteConfigError::Parse {
            path: path.clone(),
            source,
        })?;

        if site.title.trim().is_empty() {
            return Err(OgpSiteConfigError::EmptyTitle { path });
        }

        Ok(site)
    }
}

/// バリデーションを行なっていない型
/// Rawがついているのは未バリデーション
#[derive(Debug, Clone, Deserialize)]
struct RawConfig {
    id: String,
    slug: String,
    title: String,
    description: String,
    tags: Vec<String>,
    published_at: String,
    updated_at: String,
}

#[derive(Debug, Clone)]
struct RawArticleInput {
    root: PathBuf,
    entry_point: PathBuf,
    config: RawConfig,
}

impl RawArticleInput {
    fn read_articles(root: impl AsRef<Path>) -> Result<Vec<Self>, Box<dyn std::error::Error>> {
        let root = root.as_ref();
        let articles_path = root.join(ARTICLES_PATH);
        let article_roots = fs::read_dir(articles_path)?
            .map(|entry| {
                let entry = entry?;
                let is_directory = entry.file_type()?.is_dir();
                Ok((entry.path(), is_directory))
            })
            .collect::<Result<Vec<_>, std::io::Error>>()?
            .into_iter()
            .filter_map(|(path, is_directory)| is_directory.then_some(path))
            .collect::<Vec<_>>();

        article_roots
            .into_iter()
            .map(|article_root| {
                let entry_point = article_root.join(ENTRY_POINT);
                let config_path = article_root.join(CONFIG_NAME);
                let config_contents = fs::read_to_string(config_path)?;
                let config = toml::from_str(&config_contents)?;

                Ok(Self {
                    root: PathBuf::from(root),
                    entry_point,
                    config,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArticleInput {
    root: PathBuf,
    entry_point: PathBuf,
    config: Config,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    id: String,
    slug: Slug,
    title: Title,
    description: Description,
    tags: Vec<Tag>,
    published_at: PublishedAt,
    updated_at: Option<DateTime<FixedOffset>>,
}

impl RawArticleInput {
    fn validate_all(
        rows: impl IntoIterator<Item = Self>,
    ) -> Result<Vec<ArticleInput>, ArticleValidationError> {
        let mut seen_slugs = HashSet::new();
        rows.into_iter()
            .map(|row| row.validate(&mut seen_slugs))
            .collect()
    }

    fn validate(
        self,
        seen_slugs: &mut HashSet<Slug>,
    ) -> Result<ArticleInput, ArticleValidationError> {
        let config = Config::try_from(self.config)?;
        if !seen_slugs.insert(config.slug.clone()) {
            return Err(ArticleValidationError::DuplicateSlug(config.slug.0.clone()));
        }

        Ok(ArticleInput {
            root: self.root,
            entry_point: self.entry_point,
            config,
        })
    }
}

impl TryFrom<RawConfig> for Config {
    type Error = ArticleValidationError;

    fn try_from(row: RawConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            slug: Slug::new(row.slug)?,
            title: Title::new(row.title)?,
            description: Description::new(row.description)?,
            tags: row.tags.into_iter().map(Tag).collect(),
            published_at: PublishedAt::new(row.published_at)?,
            updated_at: parse_optional_datetime("updated_at", row.updated_at)?,
        })
    }
}

fn parse_optional_datetime(
    field: &'static str,
    value: String,
) -> Result<Option<DateTime<FixedOffset>>, ArticleValidationError> {
    if value.is_empty() {
        return Ok(None);
    }

    DateTime::<FixedOffset>::parse_from_rfc3339(&value)
        .map(Some)
        .map_err(|source| ArticleValidationError::InvalidDateTime { field, source })
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Slug(String);

impl Slug {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        let is_valid = value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        });
        if !is_valid {
            return Err(ArticleValidationError::InvalidSlug(value));
        }

        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Title(String);

impl Title {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        validate_text_length("title", &value, MAX_TITLE_CHARACTERS)?;
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Description(String);

impl Description {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        validate_text_length("description", &value, MAX_DESCRIPTION_CHARACTERS)?;
        Ok(Self(value))
    }
}

fn validate_text_length(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), ArticleValidationError> {
    let actual = value.chars().count();
    if actual == 0 {
        return Err(ArticleValidationError::EmptyValue { field });
    }
    if actual > maximum {
        return Err(ArticleValidationError::TextTooLong {
            field,
            maximum,
            actual,
        });
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Tag(String);

#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishedAt(DateTime<FixedOffset>);

impl PublishedAt {
    fn new(value: String) -> Result<Self, ArticleValidationError> {
        DateTime::<FixedOffset>::parse_from_rfc3339(&value)
            .map(Self)
            .map_err(|source| ArticleValidationError::InvalidDateTime {
                field: "published_at",
                source,
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CompiledArticle {
    config: Config,
    postprocess: HtmlPostprocessOutput,
}

#[derive(Debug, Clone)]
struct Article {
    metadata: ArticleOutput,
    ogp_png: Vec<u8>,
    body_html: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ArticleOutput {
    id: String,
    slug: String,
    title: String,
    description: String,
    published_at: String,
    updated_at: Option<String>,
    tags: Vec<String>,
    url: String,
    ogp_image: OgpImageMeta,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OgpImageMeta {
    path: String,
    width: u32,
    height: u32,
    mime_type: String,
    alt: String,
}

impl Article {
    /// ファイルに書き込み
    fn write(self, root: impl AsRef<Path>) -> Result<(), ArticleWriteError> {
        let root = root.as_ref();
        let article_directory = root.join(&self.metadata.slug);
        let generated_root = root
            .parent()
            .ok_or_else(|| ArticleWriteError::MissingOutputParent(root.to_path_buf()))?;
        let ogp_directory = generated_root
            .join(PUBLIC_ROOT_DIRECTORY)
            .join(OGP_ARTICLES_DIRECTORY);
        let body_html_path = article_directory.join(BODY_HTML_FILE);
        let metadata_path = article_directory.join(ARTICLE_METADATA_FILE);
        let ogp_path = ogp_directory.join(format!("{}.png", self.metadata.slug));
        let metadata_json = serde_json::to_string_pretty(&self.metadata).map_err(|source| {
            ArticleWriteError::SerializeMetadata {
                slug: self.metadata.slug.clone(),
                source,
            }
        })?;
        let body_html = format!("{}\n", self.body_html);
        let metadata_json = format!("{metadata_json}\n");

        create_output_directory(&article_directory)?;
        create_output_directory(&ogp_directory)?;
        write_output_file(&body_html_path, body_html.as_bytes())?;
        write_output_file(&metadata_path, metadata_json.as_bytes())?;
        write_output_file(&ogp_path, &self.ogp_png)?;

        Ok(())
    }
}

fn create_output_directory(path: &Path) -> Result<(), ArticleWriteError> {
    fs::create_dir_all(path).map_err(|source| ArticleWriteError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn write_output_file(path: &Path, contents: &[u8]) -> Result<(), ArticleWriteError> {
    fs::write(path, contents).map_err(|source| ArticleWriteError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HtmlPostprocessOutput {
    body_html: String,
    warnings: Vec<ArticleCompileWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArticleCompileWarning {
    url: String,
    reason: String,
}

fn compile(
    article: &ArticleInput,
    postprocessors: &postprocess::Pipeline,
) -> Result<CompiledArticle, ArticleCompileError> {
    let html =
        typst_html_export::compile_html(&article.root, &article.entry_point).map_err(|source| {
            ArticleCompileError::TypstHtml {
                slug: article.config.slug.0.clone(),
                source,
            }
        })?;
    let postprocess = postprocessors.process(html, &article.config.slug.0)?;

    Ok(CompiledArticle {
        config: article.config.clone(),
        postprocess,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use tempfile::tempdir;

    use super::{ARTICLES_PATH, CONFIG_NAME, ENTRY_POINT, RawArticleInput};

    #[test]
    /// ファイルが読めることを確認
    fn reads_each_article_directory_in_sorted_order() {
        let root = tempdir().expect("temporary directory should be created");
        let articles_root = root.path().join(ARTICLES_PATH);
        write_article_config(&articles_root, "z-article", "z-article");
        write_article_config(&articles_root, "a-article", "a-article");
        fs::write(articles_root.join("README.md"), "article index")
            .expect("non-directory entry should be written");
        fs::create_dir_all(articles_root.join("a-article/assets"))
            .expect("nested asset directory should be created");

        let articles =
            RawArticleInput::read_articles(root.path()).expect("article inputs should be read");

        assert_eq!(articles.len(), 2);
        assert_eq!(
            articles[0].entry_point,
            articles_root.join("a-article").join(ENTRY_POINT)
        );
        assert_eq!(articles[0].config.slug, "a-article");
        assert_eq!(
            articles[1].entry_point,
            articles_root.join("z-article").join(ENTRY_POINT)
        );
        assert_eq!(articles[1].config.slug, "z-article");
    }

    fn write_article_config(articles_root: &Path, directory: &str, slug: &str) {
        let article_root = articles_root.join(directory);
        fs::create_dir_all(&article_root).expect("article directory should be created");
        fs::write(
            article_root.join(CONFIG_NAME),
            format!(
                "id = \"019f88bd-06ad-76e0-8a70-394cb553d1af\"\nslug = \"{slug}\"\ntitle = \"Rust Article\"\ndescription = \"An article about Rust.\"\ntags = [\"rust\", \"compiler\"]\npublished_at = \"2026-10-11T12:00:00+09:00\"\nupdated_at = \"\"\n"
            ),
        )
        .expect("config should be written");
    }
}
