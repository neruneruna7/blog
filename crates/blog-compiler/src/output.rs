pub mod article;
pub mod error;
pub mod site;

pub const OUTPUT_ROOT: &str = "generated";

use std::{fs, path::Path};

use serde::Serialize;

use self::error::Error;

fn create_directory(path: &Path) -> Result<(), Error> {
    fs::create_dir_all(path).map_err(|source| Error::CreateDirectory {
        path: path.to_path_buf(),
        source,
    })
}

fn write_file(path: &Path, contents: &[u8]) -> Result<(), Error> {
    fs::write(path, contents).map_err(|source| Error::WriteFile {
        path: path.to_path_buf(),
        source,
    })
}

fn write_json(path: &Path, value: &(impl Serialize + ?Sized)) -> Result<(), Error> {
    let mut contents = serde_json::to_vec_pretty(value).map_err(|source| Error::SerializeJson {
        path: path.to_path_buf(),
        source,
    })?;
    contents.push(b'\n');
    write_file(path, &contents)
}
