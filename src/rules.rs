//! The [`rules::Rule`] type is how this crate modifies URLs. A [`rules::Rule`] contains a [`rules::conditions::Condition`] and a [`rules::mappers::Mapper`].
//! If the condition passes (returns `Ok(true)`), then the mapper is applied to a mutable reference to the URL.

use url::Url;
#[cfg(feature = "default-rules")]
use std::sync::OnceLock;
use std::fs::read_to_string;
use std::path::Path;
use std::ops::{Deref, DerefMut};
use std::borrow::Cow;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use thiserror::Error;

/// The logic for when to modify a URL.
pub mod conditions;
/// The logic for how to modify a URL.
pub mod mappers;
use crate::types;

/// The core unit describing when and how URLs are modified.
/// # Examples
/// ```
/// # use url_cleaner::rules::{Rule, conditions, mappers};
/// # use url::Url;
/// # use std::collections::HashMap;
/// assert!(Rule::Normal{condition: conditions::Condition::Never, mapper: mappers::Mapper::None}.apply(&mut Url::parse("https://example.com").unwrap()).is_err());
/// ```
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Rule {
    /// A faster but slightly less versatile mode that uses a hashmap to save on iterations in [`Rules`].
    HostMap(HashMap<String, mappers::Mapper>),
    /// The basic condition mapper rule type.
    #[serde(untagged)]
    Normal {
        /// The condition under which the provided URL is modified.
        condition: conditions::Condition,
        /// The mapper used to modify the provided URL.
        mapper: mappers::Mapper
    }
}

/// The errors that [`Rule`] can return.
#[derive(Error, Debug)]
pub enum RuleError {
    /// The URL does not meet the rule's condition.
    #[error("The URL does not meet the rule's condition.")]
    FailedCondition,
    /// The condition returned an error.
    #[error(transparent)]
    ConditionError(#[from] conditions::ConditionError),
    /// The mapper returned an error.
    #[error(transparent)]
    MapperError(#[from] mappers::MapperError),
    /// Returned when the provided URL doesn't have a host to find in a [`Rule::HostMap`].
    #[error("The provided URL doesn't have a host to find in the hashmap.")]
    UrlHasNoHost,
    /// Returned when the provided URL's host isn't in a [`Rule::HostMap`].
    #[error("The provided URL's host was not found in the hashmap.")]
    HostNotInMap
}

impl Rule {
    /// Apply the rule to the url in-place.
    /// # Errors
    /// If the call to [`Self::apply_with_config`] returns an error, that error is returned.
    pub fn apply(&self, url: &mut Url) -> Result<(), RuleError> {
        self.apply_with_config(url, &types::RuleConfig::default())
    }

    /// Apply the rule to the url in-place.
    /// # Errors
    /// If the rule is a [`Self::NormalRule`] and the contained [`NormalRule`] returns an error, that error is returned.
    /// If the rule is a [`Self::HostMap`] and the provided URL doesn't have a host, returns the error [`RuleError::UrlHasNoHost`].
    /// If the rule is a [`Self::HostMap`] and the provided URL's host isn't in the rule's map, returns the error [`RuleError::HostNotInMap`].
    pub fn apply_with_config(&self, url: &mut Url, config: &types::RuleConfig) -> Result<(), RuleError> {
        match self {
            Self::Normal{condition, mapper} => if condition.satisfied_by_with_config(url, config)? {
                mapper.apply(url)?;
                Ok(())
            } else {
                Err(RuleError::FailedCondition)
            },
            Self::HostMap(map) => Ok(map.get(url.host_str().ok_or(RuleError::UrlHasNoHost)?).ok_or(RuleError::HostNotInMap)?.apply(url)?)
        }
    }
}

/// The rules loaded into URL Cleaner at compile time.
/// When the `minify-included-strings` is enabled, the macro [`const_str::squish`] is used to squish all ASCII whitespace in the file to one space.
/// If there is more than one space in a string in part of a rule, this may mess that up.
/// `{"x":     "y"}` is compressed but functionally unchanged, but `{"x   y": "z"}` will be converted to `{"x y": "z"}`, which could alter the functionality of the rule.
/// If you cannot avoid multiple spaces in a strng then turn off the `minify-default-strings` feature to disable this squishing.
#[cfg(all(feature = "default-rules", feature = "minify-included-strings"))]
pub static RULES_STR: &str=const_str::squish!(include_str!("../default-rules.json"));
/// The non-minified rules loaded into URL Cleaner at compile time.
#[cfg(all(feature = "default-rules", not(feature = "minify-included-strings")))]
pub static RULES_STR: &str=include_str!("../default-rules.json");
/// The container for caching the parsed version of [`RULES_STR`].
#[cfg(feature = "default-rules")]
pub static RULES: OnceLock<Rules>=OnceLock::new();

/// Gets the rules compiled into the URL Cleaner binary.
/// On the first call, it parses [`RULES_STR`] and caches it in [`RULES`]. On all future calls it simply returns the cached value.
/// In the future it would be nice to have the rules pre-parsed so the startup speed can be significantly lowered, but that's pending const heap allocations and serde support.
/// # Errors
/// If the default rules cannot be parsed, returns the error [`GetRulesError::CantParseDefaultRules`].
/// If URL Cleaner was compiled without default rules, returns the error [`GetRulesError::NoDefaultRules`].
pub fn get_default_rules() -> Result<&'static Rules, GetRulesError> {
    #[cfg(feature = "default-rules")]
    {
        if let Some(rules) = RULES.get() {
            Ok(rules)
        } else {
            let rules=serde_json::from_str(RULES_STR).map_err(GetRulesError::CantParseDefaultRules)?;
            Ok(RULES.get_or_init(|| rules))
        }
    }
    #[cfg(not(feature = "default-rules"))]
    Err(GetRulesError::NoDefaultRules)
}

/// If `path` is `Some`, loads and parses [`Rules`] from the JSON file it points to.
/// If `None`, returns [`get_default_rules`].
/// # Errors
/// If `path` is `None` and [`get_default_rules`] returns an error, that error is returned.
/// If the specified file can't be loaded, returns the error [`GetRulesError::CantLoadFile`].
/// If the rules contained in the specified file can't be parsed, returns the error [`GetRulesError::CantParseFile`].
pub fn get_rules(path: Option<&Path>) -> Result<Cow<Rules>, GetRulesError> {
    Ok(match path {
        Some(path) => Cow::Owned(serde_json::from_str::<Rules>(&read_to_string(path).or(Err(GetRulesError::CantLoadFile))?)?),
        None => Cow::Borrowed(get_default_rules()?)
    })
}

/// A thin wrapper around a vector of rules.
/// Exists mainly for convenience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rules(Vec<Rule>);

impl From<Vec<Rule>> for Rules {fn from(value: Vec<Rule>) -> Self {Self(value)}}
impl From<Rules> for Vec<Rule> {fn from(value: Rules    ) -> Self {value.0    }}

impl Deref for Rules {
    type Target = Vec<Rule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Rules {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[allow(dead_code)]
impl Rules {
    /// A wrapper around [`Rules::deref`]
    #[must_use]
    pub fn as_slice(&self) -> &[Rule] {self}
    /// A wrapper around [`Rules::deref_mut`]
    pub fn as_mut_slice(&mut self) -> &mut [Rule] {self}

    /// Applies each rule to the provided [`Url`] in order.
    /// Bubbles up every unignored error except for [`RuleError::FailedCondition`].
    /// If an error is returned, `url` is left unmodified.
    /// # Errors
    /// If a rule returns any error other than [`RuleError::FailedCondition`], that error is returned.
    /// If the error [`RuleError::FailedCondition`] is encountered, it is ignored.
    pub fn apply(&self, url: &mut Url) -> Result<(), RuleError> {
        self.apply_with_config(url, &types::RuleConfig::default())
    }

    /// Applies each rule to the provided [`Url`] in order.
    /// Bubbles up every unignored error except for [`RuleError::FailedCondition`].
    /// If an error is returned, `url` is left unmodified.
    /// # Errors
    /// If the error [`RuleError::FailedCondition`], [`RuleError::UrlHasNoHost`], or [`RuleError::HostNotInMap`] is encountered, it is ignored.
    pub fn apply_with_config(&self, url: &mut Url, config: &types::RuleConfig) -> Result<(), RuleError> {
        let mut temp_url=url.clone();
        for rule in &**self {
            match rule.apply_with_config(&mut temp_url, config) {
                Err(RuleError::FailedCondition | RuleError::UrlHasNoHost | RuleError::HostNotInMap) => {},
                e @ Err(_) => e?,
                _ => {}
            }
        }
        *url=temp_url;
        Ok(())
    }
}

/// An enum containing all possible errors that can happen when loading/parsing a rules into a [`Rules`]
#[derive(Error, Debug)]
pub enum GetRulesError {
    /// Could not load the specified rules file.
    #[error("Could not load the specified rules file.")]
    CantLoadFile,
    /// The loaded file did not contain valid JSON.
    #[error(transparent)]
    CantParseFile(#[from] serde_json::Error),
    /// URL Cleaner was compiled without default rules.
    #[allow(dead_code)]
    #[error("URL Cleaner was compiled without default rules.")]
    NoDefaultRules,
    /// The default rules compiled into URL Cleaner aren't valid JSON.
    #[allow(dead_code)]
    #[error("The default rules compiled into URL Cleaner aren't valid JSON.")]
    CantParseDefaultRules(serde_json::Error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "default-rules")]
    fn parse_default_rules() {
        assert!(get_default_rules().is_ok());
    }
}
