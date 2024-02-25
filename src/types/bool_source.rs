use serde::{Serialize, Deserialize};
use thiserror::Error;
use url::Url;

use super::*;
use crate::glue::string_or_struct;
use crate::config::Params;

/// Various possible ways to get a boolean value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BoolSource {
    // Debug/constants.

    /// Always returns `true`.
    Always,
    /// Always returns `false`.
    Never,
    /// Always returns the error [`StringMatcherError::ExplicitError`].
    /// # Errors
    /// Always returns the error [`StringMatcherError::ExplicitError`].
    Error,
    /// Prints debugging information about the contained [`Self`] and the details of its execution to STDERR.
    /// Intended primarily for debugging logic errors.
    /// *Can* be used in production as in both bash and batch `x | y` only pipes `x`'s STDOUT, but you probably shouldn't.
    /// # Errors
    /// If the contained [`Self`] errors, returns that error.
    Debug(Box<Self>),

    // Conditional.

    /// If `r#if` passes, return the result of `then`, otherwise return the value of `r#else`.
    /// # Errors
    /// If `r#if` returns an error, that error is returned.
    /// If `r#if` passes and `then` returns an error, that error is returned.
    /// If `r#if` fails and `r#else` returns an error, that error is returned.
    If {
        /// The [`Self`] that decides if `then` or `r#else` is used.
        r#if: Box<Self>,
        /// The [`Self`] to use if `r#if` passes.
        then: Box<Self>,
        /// The [`Self`] to use if `r#if` fails.
        r#else: Box<Self>
    },
    /// Passes if the included [`Self`] doesn't and vice-versa.
    /// # Errors
    /// If the contained [`Self`] returns an error, that error is returned.
    Not(Box<Self>),
    /// Passes if all of the included [`Self`]s pass.
    /// Like [`Iterator::all`], an empty list passes..
    /// # Errors
    /// If any contained [`Self`] returns an error, that error is returned.
    All(Vec<Self>),
    /// Passes if any of the included [`Self`]s pass.
    /// Like [`Iterator::any`], an empty list fails..
    /// # Errors
    /// If any contained [`Self`] returns an error, that error is returned.
    Any(Vec<Self>),

    // Error handling.

    /// If the contained [`Self`] returns an error, treat it as a pass.
    TreatErrorAsPass(Box<Self>),
    /// If the contained [`Self`] returns an error, treat it as a fail.
    TreatErrorAsFail(Box<Self>),
    /// If `try` returns an error, `else` is executed.
    /// If `try` does not return an error, `else` is not executed.
    /// # Errors
    /// If `else` returns an error, that error is returned.
    TryElse {
        /// The [`Self`] to try first.
        r#try: Box<Self>,
        /// If `try` fails, instead return the result of this one.
        r#else: Box<Self>
    },

    // Non-meta.

    /// Get two strings them compare them.
    /// Passes if the comparison returns `true`.
    /// # Errors
    /// If either `l` or `r` return an error, that error is returned.
    /// If either `l` or `r` return `None` because the respective `none_to_empty_string` is `false`, returns the error [`BoolSourceError::StringSourceIsNone`].
    #[cfg(all(feature = "string-source", feature = "string-cmp"))]
    StringCmp {
        /// The source of the left hand side of the comparison.
        #[serde(deserialize_with = "string_or_struct")]
        l: StringSource,
        /// The source of the right hand side of the comparison.
        #[serde(deserialize_with = "string_or_struct")]
        r: StringSource,
        /// If `l` returns `None` and this is `true`, pretend `l` returned `Some("")`.
        #[serde(default = "get_true")]
        l_none_to_empty_string: bool,
        /// If `r` returns `None` and this is `true`, pretend `r` returned `Some("")`.
        #[serde(default = "get_true")]
        r_none_to_empty_string: bool,
        /// How to compare the strings from `l` and `r`.
        cmp: StringCmp
    },
    /// Checks if `needle` exists in `haystack` according to `location`.
    /// # Errors
    /// If either `haystack`'s or `needle`;s call to [`StringSource::get`] returns `None` because their respective `none_to_empty_string` is `false`, returns the error [`BoolSourceError::StringSourceIsNone`].
    /// If the call to [`StringLocation::satisfied_by`] returns an error, that error is returned.
    #[cfg(all(feature = "string-source", feature = "string-location"))]
    StringLocation {
        /// The haystack to search for `needle` in.
        #[serde(deserialize_with = "string_or_struct")]
        haystack: StringSource,
        /// The needle to search for in `haystack`.
        #[serde(deserialize_with = "string_or_struct")]
        needle: StringSource,
        /// Decides if `haystack`'s call to [`StringSource::get`] should return `Some("")` instead of `None`.
        /// Defaults to `true`.
        #[serde(default = "get_true")]
        haystack_none_to_empty_string: bool,
        /// Decides if `needle`'s call to [`StringSource::get`] should return `Some("")` instead of `None`.
        /// Defaults to `true`.
        #[serde(default = "get_true")]
        needle_none_to_empty_string: bool,
        /// The location to search for `needle` at in `haystack`.
        location: StringLocation
    },
    /// Checks if `string` matches `matcher`.
    /// # Errors
    /// If `string`'s call to [`StringSource::get`] returns `None` because `none_to_empty_string` is `false`, returns the error [`BoolSourceError::StringSourceIsNone`].
    #[cfg(all(feature = "string-source", feature = "string-matcher"))]
    StringMatcher {
        /// The string to match against.
        #[serde(deserialize_with = "string_or_struct")]
        string: StringSource,
        /// Decides if `string`'s call to [`StringSource::get`] should return `Some("")` instead of `None`.
        /// Defaults to `true`.
        #[serde(default = "get_true")]
        none_to_empty_string: bool,
        /// The matcher to check `string` against.
        matcher: StringMatcher
    },
    /// Checks if the specified flag is set.
    #[cfg(feature = "string-source")]
    FlagIsSet(#[serde(deserialize_with = "string_or_struct")] StringSource),
    /// Checks if the specified flag is set.
    #[cfg(not(feature = "string-source"))]
    FlagIsSet(String)
}

const fn get_true() -> bool {true}

/// The enum of all possible errors [`BoolSource::get`] can return.
#[derive(Debug, Error)]
pub enum BoolSourceError {
    /// Returned when [`BoolSource::Error`] is used.
    #[error("BoolSource::Error was used.")]
    ExplicitError,
    /// Returned when a [`StringSourceError`] is encountered.
    #[cfg(feature = "string-source")]
    #[error(transparent)]
    StringSourceError(#[from] StringSourceError),
    /// Returned when a [`StringLocationError`] is encountered.
    #[cfg(feature = "string-location")]
    #[error(transparent)]
    StringLocationError(#[from] StringLocationError),
    /// Returned when a [`StringMatcherError`] is encountered.
    #[cfg(feature = "string-matcher")]
    #[error(transparent)]
    StringMatcherError(#[from] StringMatcherError),
    /// Returned when a call to [`StringSource::get`] returns `None` where it has to be `Some`.
    #[cfg(feature = "string-source")]
    #[error("The specified StringSource returned None where it had to be Some.")]
    StringSourceIsNone
}

impl BoolSource {
    /// # Errors
    /// See [`Self`]'s documentation for details.
    pub fn get(&self, url: &Url, params: &Params) -> Result<bool, BoolSourceError> {
        Ok(match self {
            // Debug/constants.

            Self::Always => true,
            Self::Never => false,
            Self::Error => Err(BoolSourceError::ExplicitError)?,
            Self::Debug(bool_source) => {
                let ret=bool_source.get(url, params);
                eprintln!("=== BoolSource::Debug ===\nBoolSource: {bool_source:?}\nURL: {url:?}\nParams: {params:?}\nRet: {ret:?}");
                ret?
            },

            // Conditional.

            Self::If {r#if, then, r#else} => if r#if.get(url, params)? {then} else {r#else}.get(url, params)?,
            Self::Not(bool_source) => !bool_source.get(url, params)?,
            Self::All(bool_sources) => {
                for bool_source in bool_sources {
                    if !bool_source.get(url, params)? {
                        return Ok(false);
                    }
                }
                true
            },
            Self::Any(bool_sources) => {
                for bool_source in bool_sources {
                    if bool_source.get(url, params)? {
                        return Ok(true);
                    }
                }
                false
            },

            // Error handling.

            Self::TreatErrorAsPass(bool_source) => bool_source.get(url, params).unwrap_or(true),
            Self::TreatErrorAsFail(bool_source) => bool_source.get(url, params).unwrap_or(false),
            Self::TryElse {r#try, r#else} => r#try.get(url, params).or_else(|_| r#else.get(url, params))?,

            // Non-meta.

            #[cfg(feature = "string-source")]
            Self::StringCmp {l, r, l_none_to_empty_string, r_none_to_empty_string, cmp} => cmp.satisfied_by(
                &l.get(url, params, *l_none_to_empty_string)?.ok_or(BoolSourceError::StringSourceIsNone)?,
                &r.get(url, params, *r_none_to_empty_string)?.ok_or(BoolSourceError::StringSourceIsNone)?
            ),
            #[cfg(all(feature = "string-source", feature = "string-location"))]
            Self::StringLocation {haystack, needle, haystack_none_to_empty_string, needle_none_to_empty_string, location} => location.satisfied_by(
                &haystack.get(url, params, *haystack_none_to_empty_string)?.ok_or(BoolSourceError::StringSourceIsNone)?,
                &needle  .get(url, params, *needle_none_to_empty_string  )?.ok_or(BoolSourceError::StringSourceIsNone)?
            )?,
            #[cfg(all(feature = "string-source", feature = "string-matcher"))]
            Self::StringMatcher {string, none_to_empty_string, matcher} => matcher.satisfied_by(
                &string.get(url, params, *none_to_empty_string)?.ok_or(BoolSourceError::StringSourceIsNone)?,
                url, params
            )?,
            #[cfg(feature = "string-source")]
            Self::FlagIsSet(name) => params.flags.contains(&name.get(url, params, false)?.ok_or(BoolSourceError::StringSourceIsNone)?.into_owned()),
            #[cfg(not(feature = "string-source"))]
            Self::FlagIsSet(name) => params.flags.contains(name)
        })
    }
}
