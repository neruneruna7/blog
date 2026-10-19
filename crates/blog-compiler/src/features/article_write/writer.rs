use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::commons::{article::Slug, atomic_file::write_atomic};

use super::error::Error;

const ARTICLES_DIRECTORY: &str = "articles";
const BODY_HTML_FILE: &str = "body.html";

/// 変換済みの記事本文を生成先へ書き込む。
pub struct ArticleWriter;

impl ArticleWriter {
    /// 変換済みHTMLで記事本文を置き換える。
    ///
    /// 書き込み先は生成先の`articles/<slug>/body.html`である。
    ///
    /// # Examples
    ///
    /// ```
    /// use blog_compiler::{
    ///     commons::article::Slug,
    ///     features::article_write::ArticleWriter,
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output_root = tempfile::tempdir()?;
    /// let slug = Slug::new("example".to_owned())?;
    /// ArticleWriter::write(&slug, "<p>Example</p>", output_root.path())?;
    /// assert_eq!(
    ///     std::fs::read_to_string(output_root.path().join("articles/example/body.html"))?,
    ///     "<p>Example</p>\n",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 出力先ディレクトリを作成できない場合、またはファイルを置き換えられない場合に返す。
    pub fn write(slug: &Slug, html: &str, output_root: impl AsRef<Path>) -> Result<(), Error> {
        let path = article_body_path(output_root.as_ref(), slug);
        let mut contents = html.as_bytes().to_vec();
        contents.push(b'\n');
        write_atomic(&path, &contents).map_err(|source| Error::WriteHtml { path, source })
    }

    /// 入力から削除された記事の`body.html`を削除する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use blog_compiler::features::article_write::ArticleWriter;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output_root = tempfile::tempdir()?;
    /// ArticleWriter::remove_obsolete(&HashSet::new(), output_root.path())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 生成済み記事ディレクトリを検査できない場合、または古いHTMLを削除できない場合に返す。
    pub fn remove_obsolete(
        current_slugs: &HashSet<Slug>,
        output_root: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let directory = output_root.as_ref().join(ARTICLES_DIRECTORY);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(Error::ReadArticleDirectory {
                    path: directory,
                    source,
                });
            }
        };
        let current_slug_names = current_slugs
            .iter()
            .map(|slug| slug.as_str())
            .collect::<HashSet<_>>();

        for entry in entries {
            let entry = entry.map_err(|source| Error::ReadArticleDirectory {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            if !entry
                .file_type()
                .map_err(|source| Error::InspectArticle {
                    path: path.clone(),
                    source,
                })?
                .is_dir()
            {
                continue;
            }

            let slug = entry.file_name().to_string_lossy().into_owned();
            if !current_slug_names.contains(slug.as_str()) {
                let html_path = path.join(BODY_HTML_FILE);
                match fs::remove_file(&html_path) {
                    Ok(()) => {}
                    Err(source) if source.kind() == std::io::ErrorKind::NotFound => {}
                    Err(source) => {
                        return Err(Error::RemoveObsoleteHtml {
                            path: html_path,
                            source,
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

fn article_body_path(output_root: &Path, slug: &Slug) -> PathBuf {
    output_root
        .join(ARTICLES_DIRECTORY)
        .join(slug.as_str())
        .join(BODY_HTML_FILE)
}
