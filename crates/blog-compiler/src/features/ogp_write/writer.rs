use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use crate::commons::{article::Slug, atomic_file::write_atomic};

use super::error::Error;

const PUBLIC_ROOT_DIRECTORY: &str = "public-root";
const OGP_ARTICLES_DIRECTORY: &str = "ogp/articles";
const OGP_IMAGE_EXTENSION: &str = "png";

/// 生成したOGP画像を公開先へ書き込む。
pub struct OgpWriter;

impl OgpWriter {
    /// 生成したPNGで記事のOGP画像を置き換える。
    ///
    /// 書き込み先は生成先の`public-root/ogp/articles/<slug>.png`である。
    ///
    /// # Examples
    ///
    /// ```
    /// use blog_compiler::{
    ///     commons::article::Slug,
    ///     features::ogp_write::OgpWriter,
    /// };
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output_root = tempfile::tempdir()?;
    /// let slug = Slug::new("example".to_owned())?;
    /// OgpWriter::write(&slug, b"png", output_root.path())?;
    /// assert!(output_root
    ///     .path()
    ///     .join("public-root/ogp/articles/example.png")
    ///     .is_file());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 出力先ディレクトリを作成できない場合、または画像を置き換えられない場合に返す。
    pub fn write(slug: &Slug, png: &[u8], output_root: impl AsRef<Path>) -> Result<(), Error> {
        let path = ogp_image_path(output_root.as_ref(), slug);
        write_atomic(&path, png).map_err(|source| Error::WriteImage { path, source })
    }

    /// 入力から削除された記事のOGP画像を削除する。
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// use blog_compiler::features::ogp_write::OgpWriter;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output_root = tempfile::tempdir()?;
    /// OgpWriter::remove_obsolete(&HashSet::new(), output_root.path())?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// 生成済みOGPディレクトリを検査できない場合、または不要な画像を削除できない場合に返す。
    pub fn remove_obsolete(
        current_slugs: &HashSet<Slug>,
        output_root: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let directory = output_root
            .as_ref()
            .join(PUBLIC_ROOT_DIRECTORY)
            .join(OGP_ARTICLES_DIRECTORY);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(source) if source.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(Error::ReadOgpDirectory {
                    path: directory,
                    source,
                });
            }
        };
        let current_file_names = current_slugs
            .iter()
            .map(|slug| format!("{}.{}", slug.as_str(), OGP_IMAGE_EXTENSION))
            .collect::<HashSet<_>>();

        for entry in entries {
            let entry = entry.map_err(|source| Error::ReadOgpDirectory {
                path: directory.clone(),
                source,
            })?;
            let path = entry.path();
            if !entry
                .file_type()
                .map_err(|source| Error::InspectOgpImage {
                    path: path.clone(),
                    source,
                })?
                .is_file()
            {
                continue;
            }

            if !current_file_names.contains(&entry.file_name().to_string_lossy().into_owned()) {
                fs::remove_file(&path)
                    .map_err(|source| Error::RemoveObsoleteImage { path, source })?;
            }
        }

        Ok(())
    }
}

fn ogp_image_path(output_root: &Path, slug: &Slug) -> PathBuf {
    output_root
        .join(PUBLIC_ROOT_DIRECTORY)
        .join(OGP_ARTICLES_DIRECTORY)
        .join(format!("{}.{}", slug.as_str(), OGP_IMAGE_EXTENSION))
}
