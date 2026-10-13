use std::{fs, path::PathBuf};

use serde::Deserialize;

use super::error::Error;

const ARTICLES_PATH: &str = "articles";
const ENTRY_POINT: &str = "main.typ";
const ARTICLE_CONFIG_NAME: &str = "ArticleConfig.toml";

#[derive(Debug, Clone)]
pub struct RawArticle {
    pub root: PathBuf,
    pub entry_point: PathBuf,
    pub config: RawArticleConfig,
}

impl RawArticle {
    pub(super) fn read_all(root: impl AsRef<std::path::Path>) -> Result<Vec<Self>, Error> {
        let root = root.as_ref();
        let articles_path = root.join(ARTICLES_PATH);
        let mut article_roots = fs::read_dir(&articles_path)
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
                let is_directory = entry
                    .file_type()
                    .map_err(|source| Error::InspectDirectoryEntry {
                        path: path.clone(),
                        source,
                    })?
                    .is_dir();

                Ok((path, is_directory))
            })
            .collect::<Result<Vec<_>, Error>>()?
            .into_iter()
            .filter_map(|(path, is_directory)| is_directory.then_some(path))
            .collect::<Vec<_>>();
        article_roots.sort();

        article_roots
            .into_iter()
            .map(|article_root| {
                let entry_point = article_root.join(ENTRY_POINT);
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

#[derive(Debug, Clone, Deserialize)]
pub struct RawArticleConfig {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub tags: Vec<String>,
    pub published_at: String,
    pub updated_at: String,
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{ARTICLE_CONFIG_NAME, ARTICLES_PATH, ENTRY_POINT, RawArticle};

    #[test]
    /// 記事ディレクトリ名の順序で入力を返すことを確認
    fn reads_each_article_directory_in_sorted_order() {
        let root = tempdir().expect("temporary directory should be created");
        let articles_root = root.path().join(ARTICLES_PATH);
        write_article_config(&articles_root, "z-article", "z-article");
        write_article_config(&articles_root, "a-article", "a-article");
        fs::write(articles_root.join("README.md"), "article index")
            .expect("non-directory entry should be written");
        fs::create_dir_all(articles_root.join("a-article/assets"))
            .expect("nested asset directory should be created");

        let articles = RawArticle::read_all(root.path()).expect("article inputs should be read");

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

    fn write_article_config(articles_root: &std::path::Path, directory: &str, slug: &str) {
        let article_root = articles_root.join(directory);
        fs::create_dir_all(&article_root).expect("article directory should be created");
        fs::write(
            article_root.join(ARTICLE_CONFIG_NAME),
            format!(
                "id = \"019f88bd-06ad-76e0-8a70-394cb553d1af\"\nslug = \"{slug}\"\ntitle = \"Rust Article\"\ndescription = \"An article about Rust.\"\ntags = [\"rust\", \"compiler\"]\npublished_at = \"2026-10-11T12:00:00+09:00\"\nupdated_at = \"\"\n"
            ),
        )
        .expect("config should be written");
    }
}
