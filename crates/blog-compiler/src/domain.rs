use thiserror::Error;

pub mod article;
pub mod site;

#[derive(Debug, Error)]
pub enum ValueError {
    #[error("{field} must not be empty")]
    EmptyValue { field: &'static str },
    #[error("{field} must be at most {maximum} characters (got {actual})")]
    TextTooLong {
        field: &'static str,
        maximum: usize,
        actual: usize,
    },
    #[error(
        "slug must contain lowercase ASCII letters and digits separated by single hyphens: {0}"
    )]
    InvalidSlug(String),
    #[error("{field} must be RFC3339: {source}")]
    InvalidDateTime {
        field: &'static str,
        #[source]
        source: chrono::ParseError,
    },
}

fn validate_text_length(
    field: &'static str,
    value: &str,
    maximum: usize,
) -> Result<(), ValueError> {
    validate_non_empty(field, value)?;
    let actual = value.chars().count();
    if actual > maximum {
        return Err(ValueError::TextTooLong {
            field,
            maximum,
            actual,
        });
    }

    Ok(())
}

fn validate_non_empty(field: &'static str, value: &str) -> Result<(), ValueError> {
    if value.is_empty() {
        return Err(ValueError::EmptyValue { field });
    }

    Ok(())
}
