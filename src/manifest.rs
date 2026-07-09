use std::cmp::Reverse;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::article::ArticleManifest;
use crate::config::{ArticleConfig, SiteConfig};
use crate::error::{BlogError, PathContext, Result};
use crate::paths::ProjectPaths;

/// サイト全体の入力manifestである。
///
/// # なぜ実装するのか
/// SiteConfigと全記事設定を一貫した検証済み入力として扱うためである。
///
/// # 期待する動作
/// 必須入力を読み込み，slug/id重複やslug形式違反を検出する。
///
/// # 使い方の概要
/// `load_manifest(root)`で生成し，checkまたはbuildへ渡す。
#[derive(Clone, Debug)]
pub struct BuildInput {
    pub site: SiteConfig,
    pub articles: Vec<ArticleManifest>,
}

impl BuildInput {
    /// 公開対象記事だけを返す。
    ///
    /// # なぜ実装するのか
    /// draft除外をindex，feed，sitemap，記事出力で一貫させるためである。
    ///
    /// # 期待する動作
    /// draft記事を除外し，公開日時の新しい順で返す。
    ///
    /// # 使い方の概要
    /// buildとrenderで公開成果物の対象として使う。
    pub fn public_articles(&self) -> Vec<&ArticleManifest> {
        let mut articles = self
            .articles
            .iter()
            .filter(|article| article.is_public())
            .collect::<Vec<_>>();
        articles.sort_by_key(|article| Reverse(article.config.published_at));
        articles
    }
}

/// 入力ディレクトリからmanifestを読み込み検証する。
///
/// # なぜ実装するのか
/// buildとcheckが同じドメイン検証を共有し，挙動差を作らないためである。
///
/// # 期待する動作
/// SiteConfig.tomlと記事設定を読み，必須項目，日時，slug形式，一意性を検証する。
///
/// # 使い方の概要
/// CLIサブコマンドの最初に呼び出し，成功値を後続処理へ渡す。
pub fn load_manifest(root: impl AsRef<Path>) -> Result<BuildInput> {
    let paths = ProjectPaths::new(root);
    let site_path = paths.typst_src.join("SiteConfig.toml");
    if !site_path.exists() {
        return Err(BlogError::MissingInput(site_path));
    }
    let site = read_toml::<SiteConfig>(&site_path)?;

    let articles_dir = paths.articles_dir();
    if !articles_dir.exists() {
        return Err(BlogError::MissingInput(articles_dir));
    }

    let mut articles = fs::read_dir(&articles_dir)
        .at(&articles_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()))
        .filter_map(|entry| {
            let config_path = entry.path().join("ArticleConfig.toml");
            config_path.exists().then_some((entry.path(), config_path))
        })
        .map(|(dir_path, config_path)| {
            let dir_name = dir_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_owned();
            let config = read_toml::<ArticleConfig>(&config_path)?;
            validate_slug(&config.slug, &config_path)?;
            Ok(ArticleManifest {
                dir_name,
                dir_path,
                config_path,
                config,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    articles.sort_by(|a, b| a.dir_name.cmp(&b.dir_name));
    validate_unique(&articles)?;

    Ok(BuildInput { site, articles })
}

fn read_toml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).at(path)?;
    toml::from_str(&text).map_err(|source| BlogError::Toml {
        path: path.to_path_buf(),
        source,
    })
}

fn validate_slug(slug: &str, path: &Path) -> Result<()> {
    let valid = !slug.is_empty()
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && slug
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if valid {
        Ok(())
    } else {
        Err(BlogError::InvalidSlug {
            slug: slug.to_owned(),
            path: path.to_path_buf(),
        })
    }
}

fn validate_unique(articles: &[ArticleManifest]) -> Result<()> {
    let mut ids: HashMap<_, String> = HashMap::new();
    let mut slugs: HashMap<_, String> = HashMap::new();

    for article in articles {
        if let Some(first) = ids.insert(article.config.id, article.dir_name.clone()) {
            return Err(BlogError::DuplicateId {
                id: article.config.id,
                first,
                second: article.dir_name.clone(),
            });
        }
        if let Some(first) = slugs.insert(article.config.slug.clone(), article.dir_name.clone()) {
            return Err(BlogError::DuplicateSlug {
                slug: article.config.slug.clone(),
                first,
                second: article.dir_name.clone(),
            });
        }
    }

    Ok(())
}
