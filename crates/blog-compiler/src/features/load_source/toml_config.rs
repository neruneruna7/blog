use shiguredo_toml::{Table, TomlVersion};
use thiserror::Error;

#[derive(Debug, Error)]
pub(super) enum ConfigError {
    #[error(transparent)]
    Toml(#[from] shiguredo_toml::Error),
    #[error("missing required field '{0}'")]
    MissingField(&'static str),
    #[error("field '{field}' must be {expected}")]
    InvalidField {
        field: &'static str,
        expected: &'static str,
    },
}

pub(super) fn parse(contents: &str) -> Result<Table, ConfigError> {
    shiguredo_toml::from_str_with_version(contents, TomlVersion::V1_1).map_err(Into::into)
}

pub(super) fn required_string(table: &Table, field: &'static str) -> Result<String, ConfigError> {
    table
        .get(field)
        .ok_or(ConfigError::MissingField(field))?
        .as_str()
        .map(str::to_owned)
        .ok_or(ConfigError::InvalidField {
            field,
            expected: "a string",
        })
}

pub(super) fn optional_bool(
    table: &Table,
    field: &'static str,
    default: bool,
) -> Result<bool, ConfigError> {
    table.get(field).map_or(Ok(default), |value| {
        value.as_bool().ok_or(ConfigError::InvalidField {
            field,
            expected: "a boolean",
        })
    })
}

pub(super) fn required_string_array(
    table: &Table,
    field: &'static str,
) -> Result<Vec<String>, ConfigError> {
    table
        .get(field)
        .ok_or(ConfigError::MissingField(field))?
        .as_array()
        .ok_or(ConfigError::InvalidField {
            field,
            expected: "an array of strings",
        })?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or(ConfigError::InvalidField {
                    field,
                    expected: "an array of strings",
                })
        })
        .collect()
}
