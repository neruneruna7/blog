use std::{
    fs,
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};

use crate::features::source_parse::{article::Article, site::Site};

use super::error::Error;

// ハッシュ対象外のコンパイラ依存が変わった場合に出力を無効化する世代番号。
pub(super) const HTML_CACHE_VERSION: &[u8] = b"html-cache-v1";
pub(super) const OGP_CACHE_VERSION: &[u8] = b"ogp-cache-v1";

pub(super) fn html_hash(article: &Article, source_root: &Path) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    update_frame(&mut hasher, HTML_CACHE_VERSION);

    let article_relative_directory = article
        .entry_point
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let article_directory = source_root.join(article_relative_directory);
    hash_tree(
        &mut hasher,
        source_root,
        &article_directory,
        Some(Path::new("ArticleConfig.toml")),
        "article",
        true,
    )?;

    hash_tree(
        &mut hasher,
        source_root,
        &source_root.join("commons/components"),
        None,
        "commons/components",
        false,
    )?;

    Ok(finalize(hasher))
}

pub(super) fn ogp_hash(
    article: &Article,
    site: &Site,
    source_root: &Path,
) -> Result<String, Error> {
    let mut hasher = Sha256::new();
    update_frame(&mut hasher, OGP_CACHE_VERSION);
    let tags = article
        .tags
        .iter()
        .map(|tag| tag.as_str())
        .collect::<Vec<_>>();
    let metadata = serde_json::to_vec(&(
        article.title.as_str(),
        article.description.as_str(),
        tags,
        site.title.as_str(),
    ))?;
    update_frame(&mut hasher, b"article-ogp-metadata");
    update_frame(&mut hasher, &metadata);

    hash_tree(
        &mut hasher,
        source_root,
        &source_root.join("commons/ogp"),
        None,
        "commons/ogp",
        false,
    )?;

    Ok(finalize(hasher))
}

fn hash_tree(
    hasher: &mut Sha256,
    source_root: &Path,
    directory: &Path,
    excluded_relative_path: Option<&Path>,
    namespace: &str,
    required: bool,
) -> Result<(), Error> {
    let mut files = Vec::new();
    if let Err(error) = collect_files(directory, directory, excluded_relative_path, &mut files) {
        if !required
            && matches!(
                &error,
                Error::ReadDirectory { source, .. }
                    if source.kind() == std::io::ErrorKind::NotFound
            )
        {
            update_frame(hasher, format!("{namespace}:missing").as_bytes());
            return Ok(());
        }
        return Err(error);
    }

    files.sort_by(|left, right| left.0.cmp(&right.0));
    update_frame(hasher, format!("{namespace}:files").as_bytes());
    update_frame(hasher, &(files.len() as u64).to_be_bytes());
    for (_, path) in files {
        let contents = fs::read(&path).map_err(|source| Error::ReadInput {
            path: path.clone(),
            source,
        })?;
        let path_from_root = path.strip_prefix(source_root).unwrap_or(&path);
        let path_name = format!("{namespace}/{}", path_to_string(path_from_root));
        update_frame(hasher, b"file");
        update_frame(hasher, path_name.as_bytes());
        update_frame(hasher, &contents);
    }

    Ok(())
}

fn collect_files(
    base_directory: &Path,
    directory: &Path,
    excluded_relative_path: Option<&Path>,
    files: &mut Vec<(String, PathBuf)>,
) -> Result<(), Error> {
    let entries = fs::read_dir(directory).map_err(|source| Error::ReadDirectory {
        path: directory.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| Error::ReadDirectory {
            path: directory.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let path_from_base = path.strip_prefix(base_directory).unwrap_or(&path);
        let file_type = entry.file_type().map_err(|source| Error::InspectInput {
            path: path.clone(),
            source,
        })?;
        if file_type.is_symlink() {
            if excluded_relative_path == Some(path_from_base) {
                continue;
            }
            return Err(Error::SymbolicLink(path));
        }
        if file_type.is_dir() {
            collect_files(base_directory, &path, excluded_relative_path, files)?;
        } else if file_type.is_file() {
            files.push((path_to_string(path_from_base), path));
        }
    }
    Ok(())
}

fn path_to_string(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn update_frame(hasher: &mut Sha256, value: &[u8]) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value);
}

fn finalize(hasher: Sha256) -> String {
    format!("{:x}", hasher.finalize())
}
