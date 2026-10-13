use chrono::{DateTime, FixedOffset};

use super::{ValueError, validate_non_empty, validate_text_length};

pub mod slug;
pub mod title;

pub use slug::Slug;
pub use title::ArticleTitle;

const MAX_DESCRIPTION_CHARACTERS: usize = 130;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct ArticleId(String);

impl ArticleId {
    pub fn new(value: String) -> Result<Self, ValueError> {
        validate_non_empty("id", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct Description(String);

impl Description {
    pub fn new(value: String) -> Result<Self, ValueError> {
        validate_text_length("description", &value, MAX_DESCRIPTION_CHARACTERS)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct Tag(String);

impl Tag {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct PublishedAt(DateTime<FixedOffset>);

impl PublishedAt {
    pub fn new(value: String) -> Result<Self, ValueError> {
        DateTime::<FixedOffset>::parse_from_rfc3339(&value)
            .map(Self)
            .map_err(|source| ValueError::InvalidDateTime {
                field: "published_at",
                source,
            })
    }

    pub fn as_datetime(&self) -> &DateTime<FixedOffset> {
        &self.0
    }

    pub fn to_rfc3339(&self) -> String {
        self.0.to_rfc3339()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct UpdatedAt(Option<DateTime<FixedOffset>>);

impl UpdatedAt {
    pub fn new(value: String) -> Result<Self, ValueError> {
        if value.is_empty() {
            return Ok(Self(None));
        }

        DateTime::<FixedOffset>::parse_from_rfc3339(&value)
            .map(Some)
            .map(Self)
            .map_err(|source| ValueError::InvalidDateTime {
                field: "updated_at",
                source,
            })
    }

    pub fn as_datetime(&self) -> Option<&DateTime<FixedOffset>> {
        self.0.as_ref()
    }

    pub fn to_rfc3339(&self) -> Option<String> {
        self.0.as_ref().map(DateTime::to_rfc3339)
    }
}
