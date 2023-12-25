pub use glob::{Pattern, MatchOptions};

use serde::{Serialize, Serializer};
use serde::{de::Error as DeError, Deserialize, Deserializer};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
/// The enabled form of the wrapper around [`glob::Pattern`] and [`glob::MatchOptions`].
/// Only the necessary methods are exposed for the sake of simplicity.
/// Note that if the `glob` feature is disabled, this struct is empty.
pub struct GlobWrapper {
    #[serde(flatten, serialize_with = "serialize_pattern", deserialize_with = "deserialize_pattern")]
    pub inner: Pattern,
    #[serde(flatten, with = "SerdeMatchOptions")]
    pub options: MatchOptions
}

#[derive(Debug, Deserialize)]
struct PatternParts {
    pattern: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(remote = "MatchOptions")]
struct SerdeMatchOptions {
    #[serde(default = "get_true" )] case_sensitive: bool,
    #[serde(default = "get_false")] require_literal_separator: bool,
    #[serde(default = "get_true" )] require_literal_leading_dot: bool,
}

fn get_true() -> bool {true}
fn get_false() -> bool {false}

fn deserialize_pattern<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Pattern, D::Error> {
    let pattern_parts: PatternParts = Deserialize::deserialize(deserializer)?;
    Pattern::new(&pattern_parts.pattern).map_err(|_| D::Error::custom(format!("Invalid glob pattern: {:?}.", pattern_parts.pattern)))
}

fn serialize_pattern<S: Serializer>(pattern: &Pattern, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(pattern.as_str())
}

impl GlobWrapper {
    /// Wrapper for `glob::Pattern::matches`.
    pub fn matches(&self, str: &str) -> bool {
        self.inner.matches_with(str, self.options.clone())
    }
}
