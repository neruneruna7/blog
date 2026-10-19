use chrono::{FixedOffset, SecondsFormat, Utc};
use serde::Serialize;

const DEFAULT_TITLE: &str = "新規記事";

#[derive(Debug, Serialize)]
pub(crate) struct ArticleConfig {
    pub(crate) slug: String,
    publish: bool,
    pub(crate) title: String,
    pub(crate) description: String,
    published_at: String,
    updated_at: String,
    pub(crate) tags: Vec<String>,
}

impl Default for ArticleConfig {
    fn default() -> Self {
        Self {
            slug: String::new(),
            publish: false,
            title: DEFAULT_TITLE.to_owned(),
            description: DEFAULT_TITLE.to_owned(),
            published_at: default_published_at(),
            updated_at: String::new(),
            tags: Vec::new(),
        }
    }
}

impl ArticleConfig {
    pub(crate) fn with_slug(slug: String) -> Self {
        Self {
            slug,
            ..Self::default()
        }
    }
}

fn default_published_at() -> String {
    let now = Utc::now();
    FixedOffset::east_opt(9 * 60 * 60).map_or_else(
        || now.to_rfc3339_opts(SecondsFormat::Secs, true),
        |offset| {
            now.with_timezone(&offset)
                .to_rfc3339_opts(SecondsFormat::Secs, false)
        },
    )
}
