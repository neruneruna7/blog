use crate::commons::{ValueError, validate_text_length};

pub const MAX_ARTICLE_TITLE_CHARACTERS: usize = 48;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct ArticleTitle(String);

impl ArticleTitle {
    pub fn new(value: String) -> Result<Self, ValueError> {
        validate_text_length("title", &value, MAX_ARTICLE_TITLE_CHARACTERS)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}
