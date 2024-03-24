use std::str::FromStr;

use glob::{Pattern, MatchOptions, PatternError};
use serde::{
    Serialize, Deserialize,
    ser::Serializer,
    de::{Error as _, Deserializer}
};

/// A wrapper around [`glob::Pattern`] and [`glob::MatchOptions`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(remote= "Self")]
pub struct GlobWrapper {
    /// The pattern used to match stuff.
    #[serde(flatten, serialize_with = "serialize_pattern", deserialize_with = "deserialize_pattern")]
    pub pattern: Pattern,
    /// The options used to choose how the pattern matches stuff.
    #[serde(flatten, with = "SerdeMatchOptions")]
    pub options: MatchOptions
}

impl FromStr for GlobWrapper {
    type Err=PatternError;

    /// Simply treats the string as a glob and defaults the config.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            pattern: Pattern::from_str(s)?,
            options: MatchOptions::default()
        })
    }
}

impl TryFrom<&str> for GlobWrapper {
    type Error = <Self as FromStr>::Err;

    fn try_from(s: &str) -> Result<Self, <Self as TryFrom<&str>>::Error> {
        Self::from_str(s)
    }
}

crate::util::string_or_struct_magic!(GlobWrapper);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "MatchOptions")]
struct SerdeMatchOptions {
    #[serde(default = "get_true" , skip_serializing_if = "is_true" )] case_sensitive: bool,
    #[serde(default = "get_false", skip_serializing_if = "is_false")] require_literal_separator: bool,
    #[serde(default = "get_true" , skip_serializing_if = "is_true" )] require_literal_leading_dot: bool,
}

// Serde helper functions
const fn get_true() -> bool {true}
const fn get_false() -> bool {false}
const fn is_true(x: &bool) -> bool {*x}
const fn is_false(x: &bool) -> bool {!*x}

fn deserialize_pattern<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Pattern, D::Error> {
    let pattern: String=Deserialize::deserialize(deserializer)?;
    Pattern::new(&pattern).map_err(D::Error::custom)
}

fn serialize_pattern<S: Serializer>(pattern: &Pattern, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(pattern.as_str())
}

impl GlobWrapper {
    /// Wrapper for `glob::Pattern::matches`.
    #[must_use]
    pub fn matches(&self, str: &str) -> bool {
        self.pattern.matches_with(str, self.options)
    }
}
