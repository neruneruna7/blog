use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

pub(crate) const MANIFEST_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildCacheManifest {
    pub(crate) schema_version: u32,
    pub(crate) articles: BTreeMap<String, ManifestArticle>,
}

impl BuildCacheManifest {
    pub(crate) fn empty() -> Self {
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            articles: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManifestArticle {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) html_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) ogp_hash: Option<String>,
}

impl ManifestArticle {
    pub(crate) fn empty() -> Self {
        Self {
            html_hash: None,
            ogp_hash: None,
        }
    }
}
