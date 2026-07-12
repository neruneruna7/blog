//! なぜ実装するのか:
//! - TOML入力ファイルの読み込み境界を集約するため。
//!
//! 期待する動作:
//! - サイト設定と記事設定を型付き構造体として読み込む。
//!
//! 使い方の概要:
//! - ビルド処理の前段で `load_site_config` と `load_article_configs` を呼び出す。

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use walkdir::WalkDir;

use crate::article::ArticleConfig;
use crate::error::BlogEngineError;
use crate::output::SiteOutput;

/// なぜ実装するのか:
/// - `site.toml` の入力契約をRust型として固定するため。
///
/// 期待する動作:
/// - TOMLのsnake_caseフィールドを受け取り、検証後にJSON出力型へ正規化する。
///
/// 使い方の概要:
/// - `load_site_config` が内部でデコードし、空文字検証を行う。
#[derive(Debug, Clone, Deserialize)]
pub struct SiteConfig {
    title: String,
    description: String,
    language: String,
    base_url: String,
}

#[derive(Debug, Clone)]
pub(crate) struct ArticleConfigFile {
    pub(crate) config: ArticleConfig,
    pub(crate) article_dir: PathBuf,
}

/// なぜ実装するのか:
/// - サイト設定をファイル境界から読み込み、出力契約へ変換するため。
///
/// 期待する動作:
/// - `typst-src/site.toml` を読み、必須フィールド不足や空文字を拒否する。
///
/// 使い方の概要:
/// - ビルド処理の最初に入力ルートを渡して呼び出す。
pub fn load_site_config(source_root: impl AsRef<Path>) -> Result<SiteOutput, BlogEngineError> {
    let path = source_root.as_ref().join("site.toml");
    let config: SiteConfig = read_toml(&path)?;
    config.try_into()
}

/// なぜ実装するのか:
/// - 記事設定の所在と公開識別子を分離して扱うため。
///
/// 期待する動作:
/// - `typst-src/articles/**/ArticleConfig.toml` を決定的な順序で読み込む。
/// - 記事ディレクトリ名はソース配置上の名前として保持し、slugをそこから推測しない。
///
/// 使い方の概要:
/// - 正規化処理の前段として、入力ルートを渡して呼び出す。
pub fn load_article_configs(
    source_root: impl AsRef<Path>,
) -> Result<Vec<ArticleConfig>, BlogEngineError> {
    load_article_config_files(source_root).map(|files| {
        files
            .into_iter()
            .map(|file| file.config)
            .collect::<Vec<ArticleConfig>>()
    })
}

pub(crate) fn load_article_config_files(
    source_root: impl AsRef<Path>,
) -> Result<Vec<ArticleConfigFile>, BlogEngineError> {
    let articles_root = source_root.as_ref().join("articles");
    if !articles_root.exists() {
        return Ok(Vec::new());
    }

    let mut paths = WalkDir::new(&articles_root)
        .into_iter()
        .filter_map(|entry| match entry {
            Ok(entry)
                if entry.file_type().is_file() && entry.file_name() == "ArticleConfig.toml" =>
            {
                Some(Ok(entry.into_path()))
            }
            Ok(_) => None,
            Err(error) => Some(Err(BlogEngineError::from(error))),
        })
        .collect::<Result<Vec<PathBuf>, BlogEngineError>>()?;

    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let config = read_toml::<ArticleConfig>(&path)?;
            let article_dir = path
                .parent()
                .expect("ArticleConfig.toml path must have a parent directory")
                .to_path_buf();

            Ok(ArticleConfigFile {
                config,
                article_dir,
            })
        })
        .collect()
}

impl TryFrom<SiteConfig> for SiteOutput {
    type Error = BlogEngineError;

    fn try_from(config: SiteConfig) -> Result<Self, Self::Error> {
        validate_non_empty("site.title", &config.title)?;
        validate_non_empty("site.language", &config.language)?;
        validate_non_empty("site.base_url", &config.base_url)?;

        Ok(Self {
            title: config.title,
            description: config.description,
            language: config.language,
            base_url: config.base_url,
        })
    }
}

pub(crate) fn validate_non_empty(field: &str, value: &str) -> Result<(), BlogEngineError> {
    if value.is_empty() {
        return Err(BlogEngineError::validation(format!(
            "{field} must not be empty"
        )));
    }
    Ok(())
}

fn read_toml<T>(path: &Path) -> Result<T, BlogEngineError>
where
    T: for<'de> Deserialize<'de>,
{
    let content = fs::read_to_string(path).map_err(|source| BlogEngineError::io(path, source))?;
    toml::from_str(&content).map_err(|source| BlogEngineError::toml(path, source))
}
