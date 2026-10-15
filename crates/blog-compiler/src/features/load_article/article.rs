use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::error::Error;

const ARTICLES_FOLDER: &str = "articles";
const ARTICLE_CONFIG_NAME: &str = "ArticleConfig.toml";

#[derive(Debug, Clone)]
pub struct RawArticle {
    pub root: PathBuf,
    pub entry_point: PathBuf,
    pub config: RawArticleConfig,
}

impl RawArticle {
    pub(super) fn read_all(
        root: impl AsRef<std::path::Path>,
        entry_point_file: impl AsRef<std::path::Path>,
    ) -> Result<Vec<Self>, Error> {
        let root = root.as_ref();
        let entry_point_file = entry_point_file.as_ref();
        let articles_path = root.join(ARTICLES_FOLDER);
        let article_roots = fs::read_dir(&articles_path)
            .map_err(|source| Error::ReadDirectory {
                path: articles_path.clone(),
                source,
            })?
            .map(|entry| {
                let entry = entry.map_err(|source| Error::ReadDirectoryEntry {
                    path: articles_path.clone(),
                    source,
                })?;
                let path = entry.path();
                let directory_name = entry.file_name();
                let is_directory = entry
                    .file_type()
                    .map_err(|source| Error::InspectDirectoryEntry {
                        path: path.clone(),
                        source,
                    })?
                    .is_dir();

                Ok((path, directory_name, is_directory))
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .filter_map(|(path, directory_name, is_directory)| {
                is_directory.then_some((path, directory_name))
            })
            .collect::<Vec<_>>();

        article_roots
            .into_iter()
            .map(|(article_root, article_directory)| {
                let entry_point = article_entry_point_path(&article_directory, entry_point_file);
                let config_path = article_root.join(ARTICLE_CONFIG_NAME);
                let config_contents = fs::read_to_string(&config_path).map_err(|source| {
                    Error::ReadArticleConfig {
                        path: config_path.clone(),
                        source,
                    }
                })?;
                let config = toml::from_str(&config_contents).map_err(|source| {
                    Error::ParseArticleConfig {
                        path: config_path,
                        source,
                    }
                })?;

                Ok(Self {
                    root: root.to_path_buf(),
                    entry_point,
                    config,
                })
            })
            .collect()
    }
}

fn article_entry_point_path(article_directory: &OsStr, entry_point_file: &Path) -> PathBuf {
    Path::new(ARTICLES_FOLDER)
        .join(article_directory)
        .join(entry_point_file)
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawArticleConfig {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub published_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        ffi::OsStr,
        fs,
        path::{Path, PathBuf},
    };

    use tempfile::tempdir;

    use crate::commons::app_config::ARTICLE_ENTRY_POINT;

    use super::{ARTICLE_CONFIG_NAME, ARTICLES_FOLDER, RawArticle, article_entry_point_path};

    #[test]
    /// ディレクトリの列挙順に依存せず各記事入力を読むことを確認
    fn reads_each_article_directory_without_requiring_order() {
        let root = tempdir().expect("temporary directory should be created");
        let articles_root = root.path().join(ARTICLES_FOLDER);
        write_article_config(&articles_root, "z-article", "z-article");
        write_article_config(&articles_root, "a-article", "a-article");
        fs::write(articles_root.join("README.md"), "article index")
            .expect("non-directory entry should be written");
        fs::create_dir_all(articles_root.join("a-article/assets"))
            .expect("nested asset directory should be created");

        let articles = RawArticle::read_all(root.path(), ARTICLE_ENTRY_POINT)
            .expect("article inputs should be read");

        let slugs = articles
            .iter()
            .map(|article| article.config.slug.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(slugs, HashSet::from(["a-article", "z-article"]));

        let entry_points = articles
            .into_iter()
            .map(|article| article.entry_point)
            .collect::<HashSet<_>>();
        assert_eq!(
            entry_points,
            HashSet::from([
                PathBuf::from("articles/a-article/main.typ"),
                PathBuf::from("articles/z-article/main.typ"),
            ])
        );
    }

    #[test]
    /// 記事ディレクトリ名からプロジェクトルート基準のパスを組み立てることを確認
    fn builds_entry_point_path_relative_to_project_root() {
        let path =
            article_entry_point_path(OsStr::new("team-communication"), Path::new("main.typ"));

        assert_eq!(path, PathBuf::from("articles/team-communication/main.typ"));
    }

    fn write_article_config(articles_root: &std::path::Path, directory: &str, slug: &str) {
        let article_root = articles_root.join(directory);
        fs::create_dir_all(&article_root).expect("article directory should be created");
        fs::write(
            article_root.join(ARTICLE_CONFIG_NAME),
            format!(
                "slug = \"{slug}\"\ntitle = \"Rust Article\"\ndescription = \"An article about Rust.\"\ntags = [\"rust\", \"compiler\"]\npublished_at = \"2026-10-11T12:00:00+09:00\"\nupdated_at = \"\"\n"
            ),
        )
        .expect("config should be written");
    }
}
