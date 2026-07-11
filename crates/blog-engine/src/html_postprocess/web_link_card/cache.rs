use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::metadata::CardMetadata;

const SCHEMA_VERSION: u8 = 1;

pub fn load(cache_root: &Path, normalized_url: &str) -> Option<CardMetadata> {
    let path = entry_path(cache_root, normalized_url);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            eprintln!(
                "warning: web-link-card cache read failed at {}: {error}",
                path.display()
            );
            return None;
        }
    };
    match serde_json::from_str::<CardMetadata>(&raw) {
        Ok(metadata) if metadata.schema_version == SCHEMA_VERSION => Some(metadata),
        Ok(_) => None,
        Err(error) => {
            eprintln!(
                "warning: web-link-card cache is corrupt at {}: {error}",
                path.display()
            );
            None
        }
    }
}

pub fn store(cache_root: &Path, normalized_url: &str, metadata: &CardMetadata) {
    let path = entry_path(cache_root, normalized_url);
    let result = (|| -> Result<(), Box<dyn std::error::Error>> {
        let parent = path.parent().expect("cache entry path has a parent");
        fs::create_dir_all(parent)?;
        fs::write(path, serde_json::to_vec_pretty(metadata)?)?;
        Ok(())
    })();
    if let Err(error) = result {
        eprintln!("warning: web-link-card cache write failed: {error}");
    }
}

fn entry_path(cache_root: &Path, normalized_url: &str) -> PathBuf {
    let digest = Sha256::digest(normalized_url.as_bytes());
    cache_root
        .join("metadata")
        .join(format!("{:x}.json", digest))
}
