//! なぜ実装するのか:
//! - 記事メタデータの検証、正規化、並び順を定義するため。
//!
//! 期待する動作:
//! - ArticleConfig.tomlの値を検証し、生成JSON用の安定した記事メタデータへ変換する。
//!
//! 使い方の概要:
//! - 読み込んだ記事設定を `normalize_articles` に渡す。

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{DateTime, FixedOffset};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::config::{validate_non_empty, ArticleConfigFile};
use crate::error::BlogEngineError;

static SLUG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$")
        .expect("slug regex must be a valid regular expression")
});

/// なぜ実装するのか:
/// - `ArticleConfig.toml` の入力契約をRust型として固定するため。
///
/// 期待する動作:
/// - TOMLのsnake_caseフィールドを受け取り、検証後に記事メタデータへ正規化する。
///
/// 使い方の概要:
/// - `config::load_article_configs` がデコードし、`normalize_articles` が検証する。
#[derive(Debug, Clone, Deserialize)]
pub struct ArticleConfig {
    id: String,
    slug: String,
    title: String,
    description: String,
    published_at: String,
    updated_at: String,
    tags: Vec<String>,
}

/// なぜ実装するのか:
/// - 生成JSONで共有する記事メタデータの安定契約を表すため。
///
/// 期待する動作:
/// - serdeでcamelCase JSONへ変換され、URLとOGP画像の既定値を含む。
///
/// 使い方の概要:
/// - article-index.jsonの要素、および各記事のmeta.jsonとして書き出す。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArticleMeta {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub updated_at: Option<String>,
    pub tags: Vec<String>,
    pub url: String,
    pub ogp_image: Option<String>,
}

/// なぜ実装するのか:
/// - ソース配置と公開記事メタデータを同じ記事として扱いながら、別々に保持するため。
///
/// 期待する動作:
/// - `article_dir` は `ArticleConfig.toml` の親ディレクトリを示し、公開URLと生成出力は `meta.slug` から決まる。
///
/// 使い方の概要:
/// - `normalize_article_sources` が生成し、本文生成処理へ渡す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ArticleSource {
    pub(crate) meta: ArticleMeta,
    pub(crate) article_dir: PathBuf,
    pub(crate) main_typ_path: PathBuf,
}

/// なぜ実装するのか:
/// - 複数記事の一意性検証と決定的な並び順を一箇所に集約するため。
///
/// 期待する動作:
/// - idとslugの重複を拒否し、公開日時降順、同日時ではslug昇順で返す。
///
/// 使い方の概要:
/// - 読み込んだ `ArticleConfig` の一覧を渡し、出力用メタデータを受け取る。
pub fn normalize_articles(
    configs: Vec<ArticleConfig>,
) -> Result<Vec<ArticleMeta>, BlogEngineError> {
    let mut articles = configs
        .into_iter()
        .map(validate_article)
        .collect::<Result<Vec<_>, BlogEngineError>>()?;

    validate_unique_articles(&articles)?;
    sort_articles(&mut articles);

    Ok(articles)
}

pub(crate) fn normalize_article_sources(
    files: Vec<ArticleConfigFile>,
) -> Result<Vec<ArticleSource>, BlogEngineError> {
    let mut sources = files
        .into_iter()
        .map(|file| {
            let meta = validate_article(file.config)?;
            let main_typ_path = file.article_dir.join("main.typ");

            Ok(ArticleSource {
                meta,
                article_dir: file.article_dir,
                main_typ_path,
            })
        })
        .collect::<Result<Vec<_>, BlogEngineError>>()?;

    let metas = sources
        .iter()
        .map(|source| source.meta.clone())
        .collect::<Vec<ArticleMeta>>();
    validate_unique_articles(&metas)?;
    sources.sort_by(|left, right| compare_articles(&left.meta, &right.meta));

    Ok(sources)
}

fn validate_unique_articles(articles: &[ArticleMeta]) -> Result<(), BlogEngineError> {
    let mut seen_ids = HashSet::new();
    let mut seen_slugs = HashSet::new();

    for article in articles {
        if !seen_ids.insert(article.id.clone()) {
            return Err(BlogEngineError::validation(format!(
                "article id must be unique: {}",
                article.id
            )));
        }
        if !seen_slugs.insert(article.slug.clone()) {
            return Err(BlogEngineError::validation(format!(
                "article slug must be unique: {}",
                article.slug
            )));
        }
    }

    Ok(())
}

fn sort_articles(articles: &mut [ArticleMeta]) {
    articles.sort_by(compare_articles);
}

fn compare_articles(left: &ArticleMeta, right: &ArticleMeta) -> std::cmp::Ordering {
    let left_published_at = parse_validated_rfc3339(&left.published_at);
    let right_published_at = parse_validated_rfc3339(&right.published_at);

    right_published_at
        .cmp(&left_published_at)
        .then_with(|| left.slug.cmp(&right.slug))
        .then_with(|| left.id.cmp(&right.id))
}

/// 記事単体の値検証と、出力専用フィールドの補完を行う。
fn validate_article(config: ArticleConfig) -> Result<ArticleMeta, BlogEngineError> {
    validate_non_empty("article.id", &config.id)?;
    validate_non_empty("article.slug", &config.slug)?;
    validate_non_empty("article.title", &config.title)?;
    validate_non_empty("article.description", &config.description)?;
    validate_non_empty("article.published_at", &config.published_at)?;
    validate_slug(&config.slug)?;
    validate_rfc3339("article.published_at", &config.published_at)?;

    let updated_at = if config.updated_at.is_empty() {
        None
    } else {
        validate_rfc3339("article.updated_at", &config.updated_at)?;
        Some(config.updated_at)
    };

    let url = format!("/articles/{}/", config.slug);

    Ok(ArticleMeta {
        id: config.id,
        slug: config.slug,
        title: config.title,
        description: config.description,
        published_at: config.published_at,
        updated_at,
        tags: config.tags,
        url,
        ogp_image: None,
    })
}

/// slugの許可文字と区切り規則を検証する。
pub(crate) fn validate_slug(slug: &str) -> Result<(), BlogEngineError> {
    if !SLUG_REGEX.is_match(slug) {
        return Err(BlogEngineError::validation(format!(
            "article.slug is invalid: {slug}"
        )));
    }
    Ok(())
}

/// RFC3339日時としてパースできることを検証する。
fn validate_rfc3339(field: &str, value: &str) -> Result<(), BlogEngineError> {
    DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .map(|_| ())
        .map_err(|source| BlogEngineError::validation(format!("{field} must be RFC3339: {source}")))
}

fn parse_validated_rfc3339(value: &str) -> DateTime<FixedOffset> {
    DateTime::<FixedOffset>::parse_from_rfc3339(value)
        .expect("published_at must already be validated as RFC3339")
}
