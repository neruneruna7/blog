use chrono::{FixedOffset, SecondsFormat, Utc};
use shiguredo_toml::{Table, Value};

const DEFAULT_TITLE: &str = "新規記事";

#[derive(Debug)]
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

    pub(crate) fn to_toml_value(&self) -> Value {
        let table: Table = [
            ("slug", Value::String(self.slug.clone())),
            ("publish", Value::Boolean(self.publish)),
            ("title", Value::String(self.title.clone())),
            ("description", Value::String(self.description.clone())),
            ("published_at", Value::String(self.published_at.clone())),
            ("updated_at", Value::String(self.updated_at.clone())),
            (
                "tags",
                Value::Array(self.tags.iter().cloned().map(Value::String).collect()),
            ),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value))
        .collect();

        Value::Table(table)
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
