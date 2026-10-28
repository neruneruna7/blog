use std::{
    fs, io,
    path::{Path, PathBuf},
};

use crate::commons::app_config::ARTICLE_ENTRY_POINT;

use super::{config::ArticleConfig, error::Error};
use crate::features::load_template::template::ArticleTemplate;

const MAX_STAGING_ATTEMPTS: usize = 16;

pub(crate) struct NewArticle {
    source_root: PathBuf,
    article_dir: PathBuf,
    template: ArticleTemplate,
    config: ArticleConfig,
}

impl NewArticle {
    pub(crate) fn new(
        source_root: PathBuf,
        article_dir: PathBuf,
        template: ArticleTemplate,
        config: ArticleConfig,
    ) -> Self {
        Self {
            source_root,
            article_dir,
            template,
            config,
        }
    }

    pub(crate) fn create(self) -> Result<(), Error> {
        let temporary_dir = create_temporary_directory(&self.source_root, &self.config.slug)?;
        let result = self.write_and_commit(&temporary_dir);
        if result.is_err() {
            let _ = fs::remove_dir_all(&temporary_dir);
        }

        result
    }

    fn write_and_commit(self, temporary_dir: &Path) -> Result<(), Error> {
        let config_path = temporary_dir.join("ArticleConfig.toml");
        let config_contents = shiguredo_toml::to_string_pretty(&self.config.to_toml_value())
            .map_err(Error::SerializeArticleConfig)?;
        write_file(&config_path, config_contents.as_bytes())?;

        let main_typ_path = temporary_dir.join(ARTICLE_ENTRY_POINT);
        write_file(&main_typ_path, self.template.contents())?;

        let assets_dir = temporary_dir.join("assets");
        fs::create_dir(&assets_dir).map_err(|source| Error::CreateDirectory {
            path: assets_dir.clone(),
            source,
        })?;
        write_file(&assets_dir.join(".gitkeep"), b"")?;

        if self
            .article_dir
            .try_exists()
            .map_err(|source| Error::InspectArticlePath {
                path: self.article_dir.clone(),
                source,
            })?
        {
            return Err(Error::ArticlePathExists {
                path: self.article_dir,
            });
        }

        fs::rename(temporary_dir, &self.article_dir).map_err(|source| Error::CommitArticle {
            path: self.article_dir.clone(),
            source,
        })
    }
}

fn create_temporary_directory(source_root: &Path, slug: &str) -> Result<PathBuf, Error> {
    for suffix in 0..MAX_STAGING_ATTEMPTS {
        let path = source_root.join(format!(".new-article-{slug}-{suffix}"));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(path),
            Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(source) => {
                return Err(Error::CreateDirectory { path, source });
            }
        }
    }

    Err(Error::TemporaryDirectoryExhausted {
        path: source_root.to_path_buf(),
        attempts: MAX_STAGING_ATTEMPTS,
    })
}

fn write_file(path: &Path, contents: &[u8]) -> Result<(), Error> {
    fs::write(path, contents).map_err(|source| Error::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}
