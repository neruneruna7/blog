use crate::domain::{ValueError, validate_non_empty};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(transparent)]
pub struct SiteTitle(String);

impl SiteTitle {
    pub fn new(value: String) -> Result<Self, ValueError> {
        validate_non_empty("site title", &value)?;
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}
