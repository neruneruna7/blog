//! html変換をした後，ファイルシステムに書き込むためのデータを表す．
pub mod article;
pub mod error;
pub mod site;

pub const ARTIFACT_ROOT: &str = "generated";

use std::{fs, path::Path};

use serde::Serialize;

use self::error::ArtifactWriteError;

fn create_directory(path: &Path) -> Result<(), ArtifactWriteError> {
    fs::create_dir_all(path).map_err(|source| ArtifactWriteError::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, contents: &[u8]) -> Result<(), ArtifactWriteError> {
    fs::write(path, contents).map_err(|source| ArtifactWriteError::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn write_json(path: &Path, value: &(impl Serialize + ?Sized)) -> Result<(), ArtifactWriteError> {
    let mut contents =
        serde_json::to_vec_pretty(value).map_err(|source| ArtifactWriteError::SerializeJson {
            path: path.to_path_buf(),
            source,
        })?;
    contents.push(b'\n');
    write_file(path, &contents)
}
