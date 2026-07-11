//! なぜ実装するのか:
//! - Typst記事雛形の作成をCLIから独立した再利用可能な境界として提供するため。
//!
//! 期待する動作:
//! - 既存記事を検証し、衝突しないUUIDv7とslugを持つ記事を原子的に作成する。
//!
//! 使い方の概要:
//! - `create_article` に入力ルートとタイトル・概要を渡す。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, FixedOffset, Utc};
use serde::Serialize;
use uuid::Uuid;

use crate::article::validate_slug;
use crate::error::BlogEngineError;
use crate::{load_normalized_articles, load_site_config};

const SLUG_ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
const SLUG_LENGTH: usize = 12;
const MAX_SLUG_ATTEMPTS: usize = 16;
const MAIN_TYP: &str = "// 記事本文をここに記述する．\n\n= はじめに\n\n本文を記述する．\n";

/// なぜ実装するのか:
/// - CLI入力をファイル出力や識別子生成から分離するため。
///
/// 期待する動作:
/// - タイトルと任意の概要だけを受け取り、slugやIDを外部入力にしない。
///
/// 使い方の概要:
/// - `create_article` の第二引数として構築する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewArticleRequest {
    pub title: String,
    pub description: Option<String>,
}

/// なぜ実装するのか:
/// - 作成済み記事を後続処理へ明確に引き渡すため。
///
/// 期待する動作:
/// - 記事の永続ID、slug、確定済みディレクトリを返す。
///
/// 使い方の概要:
/// - CLIはこの値を成功メッセージとして表示する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreatedArticle {
    pub id: String,
    pub slug: String,
    pub article_dir: PathBuf,
}

#[derive(Debug, Serialize)]
struct NewArticleConfig<'a> {
    id: &'a str,
    slug: &'a str,
    title: &'a str,
    description: &'a str,
    published_at: String,
    updated_at: &'static str,
    tags: Vec<String>,
}

/// なぜ実装するのか:
/// - 本番の時刻・UUIDv7・OS乱数を用いた記事作成の公開入口を提供するため。
///
/// 期待する動作:
/// - 入力を検証し、完全な記事雛形を一時ディレクトリ経由で確定先へ作成する。
///
/// 使い方の概要:
/// - `create_article("typst-src", NewArticleRequest { .. })` として呼び出す。
pub fn create_article(
    source_root: impl AsRef<Path>,
    request: NewArticleRequest,
) -> Result<CreatedArticle, BlogEngineError> {
    let offset = FixedOffset::east_opt(9 * 60 * 60)
        .ok_or_else(|| BlogEngineError::validation("UTC+09:00 offset is invalid"))?;
    let published_at = Utc::now().with_timezone(&offset);
    create_article_with_context(
        source_root.as_ref(),
        request,
        published_at,
        Uuid::now_v7(),
        generate_slug,
    )
}

/// なぜ実装するのか:
/// - 時刻・UUID・乱数という非決定値を狭い境界へ隔離し、作成処理を決定的に検証するため。
///
/// 期待する動作:
/// - 注入値以外は本番と同じ検証・staging・確定手順で記事を作成する。
///
/// 使い方の概要:
/// - 本番は `create_article` から呼び、単体テストでは固定値を渡す。
fn create_article_with_context<F>(
    source_root: &Path,
    request: NewArticleRequest,
    published_at: DateTime<FixedOffset>,
    article_id: Uuid,
    mut next_slug: F,
) -> Result<CreatedArticle, BlogEngineError>
where
    F: FnMut() -> Result<String, BlogEngineError>,
{
    validate_source_root(source_root)?;
    load_site_config(source_root)?;
    let existing_articles = load_normalized_articles(source_root)?;
    let existing_slugs = existing_articles
        .into_iter()
        .map(|article| article.slug)
        .collect::<HashSet<_>>();
    let (title, description) = normalize_request(request)?;
    let articles_root = source_root.join("articles");
    let slug = select_unique_slug(&articles_root, &existing_slugs, &mut next_slug)?;
    let article_dir = articles_root.join(&slug);
    let id = article_id.hyphenated().to_string();

    fs::create_dir_all(&articles_root)
        .map_err(|source| BlogEngineError::io(&articles_root, source))?;
    if article_dir.exists() {
        return Err(BlogEngineError::ArticlePathExists { path: article_dir });
    }

    let temporary_dir = create_temporary_directory(&articles_root, &id)?;
    let result = write_and_commit_article(
        &temporary_dir,
        &article_dir,
        &id,
        &slug,
        &title,
        &description,
        published_at,
    );
    if result.is_err() {
        let _ = fs::remove_dir_all(&temporary_dir);
    }
    result
}

fn validate_source_root(source_root: &Path) -> Result<(), BlogEngineError> {
    if !source_root.is_dir() {
        return Err(BlogEngineError::validation(format!(
            "source root does not exist or is not a directory: {}",
            source_root.display()
        )));
    }
    let site_path = source_root.join("site.toml");
    if !site_path.is_file() {
        return Err(BlogEngineError::validation(format!(
            "site.toml does not exist: {}",
            site_path.display()
        )));
    }
    Ok(())
}

fn normalize_request(request: NewArticleRequest) -> Result<(String, String), BlogEngineError> {
    let title = request.title.trim().to_owned();
    if title.is_empty() {
        return Err(BlogEngineError::validation(
            "article.title must not be empty",
        ));
    }
    let description = match request.description {
        Some(description) => {
            let description = description.trim().to_owned();
            if description.is_empty() {
                return Err(BlogEngineError::validation(
                    "article.description must not be empty",
                ));
            }
            description
        }
        None => title.clone(),
    };
    Ok((title, description))
}

fn select_unique_slug<F>(
    articles_root: &Path,
    existing_slugs: &HashSet<String>,
    next_slug: &mut F,
) -> Result<String, BlogEngineError>
where
    F: FnMut() -> Result<String, BlogEngineError>,
{
    for _ in 0..MAX_SLUG_ATTEMPTS {
        let candidate = next_slug()?;
        validate_slug(&candidate)?;
        if !existing_slugs.contains(&candidate) && !articles_root.join(&candidate).exists() {
            return Ok(candidate);
        }
    }
    Err(BlogEngineError::validation(format!(
        "could not generate a unique article slug within {MAX_SLUG_ATTEMPTS} attempts under {}",
        articles_root.display()
    )))
}

fn create_temporary_directory(
    articles_root: &Path,
    temporary_suffix: &str,
) -> Result<PathBuf, BlogEngineError> {
    for suffix in 0..MAX_SLUG_ATTEMPTS {
        let path = articles_root.join(format!(".new-article-{temporary_suffix}-{suffix}"));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(source) => return Err(BlogEngineError::io(path, source)),
        }
    }
    Err(BlogEngineError::validation(format!(
        "could not create a temporary article directory under {}",
        articles_root.display()
    )))
}

fn write_and_commit_article(
    temporary_dir: &Path,
    article_dir: &Path,
    id: &str,
    slug: &str,
    title: &str,
    description: &str,
    published_at: DateTime<FixedOffset>,
) -> Result<CreatedArticle, BlogEngineError> {
    let config_path = temporary_dir.join("ArticleConfig.toml");
    let config = NewArticleConfig {
        id,
        slug,
        title,
        description,
        published_at: published_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, false),
        updated_at: "",
        tags: Vec::new(),
    };
    let config_contents = toml::to_string_pretty(&config).map_err(|source| {
        BlogEngineError::validation(format!("could not serialize article config: {source}"))
    })?;
    fs::write(&config_path, config_contents)
        .map_err(|source| BlogEngineError::io(&config_path, source))?;

    let main_typ_path = temporary_dir.join("main.typ");
    fs::write(&main_typ_path, MAIN_TYP)
        .map_err(|source| BlogEngineError::io(&main_typ_path, source))?;

    let assets_dir = temporary_dir.join("assets");
    fs::create_dir(&assets_dir).map_err(|source| BlogEngineError::io(&assets_dir, source))?;
    let gitkeep_path = assets_dir.join(".gitkeep");
    fs::write(&gitkeep_path, "").map_err(|source| BlogEngineError::io(&gitkeep_path, source))?;

    if article_dir.exists() {
        return Err(BlogEngineError::ArticlePathExists {
            path: article_dir.to_path_buf(),
        });
    }
    fs::rename(temporary_dir, article_dir)
        .map_err(|source| BlogEngineError::io(article_dir, source))?;

    Ok(CreatedArticle {
        id: id.to_owned(),
        slug: slug.to_owned(),
        article_dir: article_dir.to_path_buf(),
    })
}

/// なぜ実装するのか:
/// - URLやタイトルに依存しない均一なslugをOS乱数から生成するため。
///
/// 期待する動作:
/// - 指定アルファベットだけから成る12文字・60ビットのslugを返す。
///
/// 使い方の概要:
/// - `create_article` が内部で呼び出し、衝突時は別候補を生成する。
fn generate_slug() -> Result<String, BlogEngineError> {
    let mut bytes = [0_u8; SLUG_LENGTH];
    getrandom::fill(&mut bytes).map_err(BlogEngineError::random_source_error)?;
    Ok(bytes
        .into_iter()
        .map(|byte| SLUG_ALPHABET[usize::from(byte & 31)] as char)
        .collect())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use chrono::{FixedOffset, TimeZone};
    use tempfile::TempDir;
    use toml::Value;
    use uuid::Uuid;

    use super::*;

    const UUID: &str = "019bf2d9-49d3-7eb3-8a2d-a1d29f396cb4";
    const SLUG: &str = "0123456789ab";

    #[test]
    fn generates_a_complete_schema_compatible_article_skeleton() {
        let source = source_root();
        let published_at = fixed_time();
        let created = create_article_with_context(
            source.path(),
            request("  記事タイトル  ", Some("  記事概要  ")),
            published_at,
            uuid(),
            || Ok(SLUG.to_owned()),
        )
        .expect("article should be created");

        assert_eq!(
            created.article_dir,
            source.path().join("articles").join(SLUG)
        );
        assert!(created.article_dir.join("ArticleConfig.toml").is_file());
        assert!(created.article_dir.join("main.typ").is_file());
        assert!(created.article_dir.join("assets/.gitkeep").is_file());
        assert_eq!(article_directories(source.path()).len(), 1);

        let config: Value = toml::from_str(
            &fs::read_to_string(created.article_dir.join("ArticleConfig.toml"))
                .expect("config should be readable"),
        )
        .expect("config should be TOML");
        assert_eq!(config["id"].as_str(), Some(UUID));
        assert_eq!(
            Uuid::parse_str(config["id"].as_str().unwrap_or_default())
                .unwrap()
                .get_version_num(),
            7
        );
        assert_eq!(config["slug"].as_str(), Some(SLUG));
        assert_eq!(config["title"].as_str(), Some("記事タイトル"));
        assert_eq!(config["description"].as_str(), Some("記事概要"));
        assert_eq!(
            config["published_at"].as_str(),
            Some("2026-07-11T12:34:56+09:00")
        );
        assert_eq!(config["updated_at"].as_str(), Some(""));
        assert_eq!(config["tags"].as_array().map(Vec::is_empty), Some(true));
        assert_eq!(
            load_normalized_articles(source.path()).unwrap()[0].slug,
            SLUG
        );
    }

    #[test]
    fn defaults_description_to_normalized_title() {
        let source = source_root();
        let created = create_article_with_context(
            source.path(),
            request("  title  ", None),
            fixed_time(),
            uuid(),
            || Ok(SLUG.to_owned()),
        )
        .unwrap();
        let config: Value = toml::from_str(
            &fs::read_to_string(created.article_dir.join("ArticleConfig.toml")).unwrap(),
        )
        .unwrap();
        assert_eq!(config["description"].as_str(), Some("title"));
    }

    #[test]
    fn rejects_blank_title_and_explicit_blank_description_without_output() {
        for request in [request(" \t ", None), request("title", Some("\n "))] {
            let source = source_root();
            assert!(create_article_with_context(
                source.path(),
                request,
                fixed_time(),
                uuid(),
                || Ok(SLUG.to_owned())
            )
            .is_err());
            assert!(!source.path().join("articles").exists());
        }
    }

    #[test]
    fn retries_slug_collisions_without_changing_existing_content() {
        let source = source_root();
        let existing = source.path().join("articles/existing-slug");
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("sentinel"), "preserve").unwrap();
        let mut candidates = ["existing-slug", "new-slug"].into_iter();
        let created = create_article_with_context(
            source.path(),
            request("title", None),
            fixed_time(),
            uuid(),
            || Ok(candidates.next().unwrap().to_owned()),
        )
        .unwrap();
        assert_eq!(created.slug, "new-slug");
        assert_eq!(
            fs::read_to_string(existing.join("sentinel")).unwrap(),
            "preserve"
        );
    }

    #[test]
    fn existing_final_path_is_never_overwritten() {
        let source = source_root();
        let existing = source.path().join("articles").join(SLUG);
        fs::create_dir_all(&existing).unwrap();
        fs::write(existing.join("sentinel"), "preserve").unwrap();
        let error = create_article_with_context(
            source.path(),
            request("title", None),
            fixed_time(),
            uuid(),
            || Ok(SLUG.to_owned()),
        )
        .unwrap_err();
        assert!(matches!(error, BlogEngineError::Validation { .. }));
        assert_eq!(
            fs::read_to_string(existing.join("sentinel")).unwrap(),
            "preserve"
        );
    }

    #[test]
    fn staging_directory_is_not_left_when_articles_root_cannot_be_created() {
        let source = source_root();
        fs::write(source.path().join("articles"), "not a directory").unwrap();
        assert!(create_article_with_context(
            source.path(),
            request("title", None),
            fixed_time(),
            uuid(),
            || Ok(SLUG.to_owned())
        )
        .is_err());
        assert!(!source
            .path()
            .join(format!("articles/.new-article-{UUID}-0"))
            .exists());
    }

    #[test]
    fn production_slug_has_the_required_format() {
        let slug = generate_slug().unwrap();
        assert_eq!(slug.len(), SLUG_LENGTH);
        assert!(slug.bytes().all(|byte| SLUG_ALPHABET.contains(&byte)));
        validate_slug(&slug).unwrap();
    }

    fn source_root() -> TempDir {
        let source = TempDir::new().unwrap();
        fs::write(
            source.path().join("site.toml"),
            "title = 'site'\ndescription = ''\nlanguage = 'ja'\nbase_url = 'https://example.com'\n",
        )
        .unwrap();
        source
    }

    fn request(title: &str, description: Option<&str>) -> NewArticleRequest {
        NewArticleRequest {
            title: title.to_owned(),
            description: description.map(str::to_owned),
        }
    }

    fn fixed_time() -> DateTime<FixedOffset> {
        FixedOffset::east_opt(9 * 60 * 60)
            .unwrap()
            .with_ymd_and_hms(2026, 7, 11, 12, 34, 56)
            .unwrap()
    }

    fn uuid() -> Uuid {
        Uuid::parse_str(UUID).unwrap()
    }

    fn article_directories(source: &Path) -> Vec<PathBuf> {
        fs::read_dir(source.join("articles"))
            .unwrap()
            .filter_map(Result::ok)
            .filter(|entry| entry.path().is_dir())
            .map(|entry| entry.path())
            .collect()
    }
}
