use serde::{Deserialize, Serialize};

use super::ogp::WebLinkCardOgp;

pub(super) const CACHE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CacheRecord {
    pub(super) schema_version: u32,
    pub(super) url: String,
    pub(super) metadata: WebLinkCardOgp,
}
