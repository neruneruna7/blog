use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use chrono::{DateTime, FixedOffset};
use serde::Deserialize;
use thiserror::Error;

pub fn article_compile() -> Result<(), ()> {
    let article_inputs = RawArticleInput::read_articles(TYP_COMPILE_ROOT).unwrap();
    let validated_article_inputs = RawArticleInput::validate_all(article_inputs).unwrap();
    let compiled = todo!("コンパイル");

    todo!("ファイル書き込み")
}

// 複数ユースケースを横断する必要があるだろうが...
// まぁ，その時にどこに置くべきか見えてくるだろう．
const TYP_COMPILE_ROOT: &str = "typst-src";
const ARTICLES_PATH: &str = "articles";
const ENTRY_POINT: &str = "main.typ";
const CONFIG_NAME: &str = "ArticleConfig.toml";
const MAX_TITLE_CHARACTERS: usize = 48;
const MAX_DESCRIPTION_CHARACTERS: usize = 130;

#[derive(Debug, Clone, Deserialize)]
struct RawConfig {
    slug: String,
    title: String,
    description: String,
    tags: Vec<String>,
    published_at: String,
}

#[derive(Debug, Clone)]
struct RawArticleInput {
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
                    entry_point,
                    config,
                })
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ArticleInput {
    entry_point: PathBuf,
    config: Config,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Config {
    slug: Slug,
    title: Title,
    description: Description,
    tags: Vec<Tag>,
    published_at: PublishedAt,
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
            entry_point: self.entry_point,
            config,
        })
    }
}

impl TryFrom<RawConfig> for Config {
    type Error = ArticleValidationError;

    fn try_from(row: RawConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            slug: Slug::new(row.slug)?,
            title: Title::new(row.title)?,
            description: Description::new(row.description)?,
            tags: row.tags.into_iter().map(Tag).collect(),
            published_at: PublishedAt::new(row.published_at)?,
        })
    }
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
        Ok(Self(DateTime::<FixedOffset>::parse_from_rfc3339(&value)?))
    }
}

#[derive(Debug, Error)]
enum ArticleValidationError {
    #[error("{field} must not be empty")]
    EmptyValue { field: &'static str },
    #[error("{field} must be at most {maximum} characters (got {actual})")]
    TextTooLong {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },
    #[error(
        "slug must contain lowercase ASCII letters and digits separated by single hyphens: {0}"
    )]
    InvalidSlug(String),
    #[error("slug must be unique: {0}")]
    DuplicateSlug(String),
    #[error("published_at must be RFC3339: {0}")]
    InvalidPublishedAt(#[from] chrono::ParseError),
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
                "slug = \"{slug}\"\ntitle = \"Rust Article\"\ndescription = \"An article about Rust.\"\ntags = [\"rust\", \"compiler\"]\npublished_at = \"2026-10-11T12:00:00+09:00\"\n"
            ),
        )
        .expect("config should be written");
    }
}
