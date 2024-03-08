use std::str::FromStr;
use std::convert::Infallible;
use std::borrow::Cow;

use serde::{Serialize, Deserialize};
use url::Url;
use thiserror::Error;
#[cfg(all(feature = "http", not(target_family = "wasm")))]
use reqwest::header::HeaderMap;

use crate::string_or_struct_magic;
use crate::types::*;
use crate::glue::*;

/// Allows conditions and mappers to get strings from various sources without requiring different conditions and mappers for each source.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(remote = "Self")]
pub enum StringSource {
    /// Always returns the error [`StringSourceError::ExplicitError`].
    /// # Errors
    /// Always returns the error [`StringSourceError::ExplicitError`].
    Error,
    /// Prints debugging information about the contained [`Self`] and the details of its execution to STDERR.
    /// Intended primarily for debugging logic errors.
    /// *Can* be used in production as in both bash and batch `x | y` only pipes `x`'s STDOUT, but you probably shouldn't.
    /// # Errors
    /// If the contained [`Self`] returns an error, that error is returned after the debug info is printed.
    Debug(Box<Self>),
    /// Just a string. The most common variant.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringSource;
    /// # use url::Url;
    /// # use url_cleaner::types::Params;
    /// # use std::borrow::Cow;
    /// let url = Url::parse("https://example.com").unwrap();
    /// StringSource::String("abc".to_string()).get(&url, &Params::default()).is_ok_and(|x| x==Some(Cow::Borrowed("abc")));
    /// ```
    String(String),
    /// Gets the specified URL part.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringSource;
    /// # use url::Url;
    /// # use url_cleaner::types::Params;
    /// # use std::borrow::Cow;
    /// # use url_cleaner::types::UrlPart;
    /// let url = Url::parse("https://example.com").unwrap();
    /// let params = Params::default();
    /// StringSource::Part(UrlPart::Domain).get(&url, &Params::default()).is_ok_and(|x| x==Some(Cow::Borrowed("example.com")));
    /// ```
    Part(UrlPart),
    /// Gets the specified variable's value.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringSource;
    /// # use url::Url;
    /// # use url_cleaner::types::Params;
    /// # use std::borrow::Cow;
    /// # use std::collections::HashMap;
    /// let url = Url::parse("https://example.com").unwrap();
    /// let params = Params {vars: HashMap::from_iter([("abc".to_string(), "xyz".to_string())]), ..Params::default()};
    /// StringSource::Var("abc".to_string()).get(&url, &params).is_ok_and(|x| x==Some(Cow::Borrowed("xyz")));
    /// ```
    Var(String),
    /// If the flag specified by `flag` is set, return the result of `then`. Otherwise return the result of `r#else`.
    /// # Examples
    /// ```
    /// # use url_cleaner::types::StringSource;
    /// # use url::Url;
    /// # use url_cleaner::types::Params;
    /// # use std::borrow::Cow;
    /// # use url_cleaner::types::UrlPart;
    /// # use std::collections::HashSet;
    /// let url = Url::parse("https://example.com").unwrap();
    /// let params_1 = Params::default();
    /// let params_2 = Params {flags: HashSet::from_iter(["abc".to_string()]), ..Params::default()};
    /// let x = StringSource::IfFlag {flag: "abc".to_string(), then: Box::new(StringSource::String("xyz".to_string())), r#else: Box::new(StringSource::Part(UrlPart::Domain))};
    /// x.get(&url, &params_1).is_ok_and(|x| x==Some(Cow::Borrowed("example.com")));
    /// x.get(&url, &params_2).is_ok_and(|x| x==Some(Cow::Borrowed("xyz")));
    /// ```
    IfFlag {
        /// The name of the flag to check.
        flag: String,
        /// If the flag is set, use this.
        then: Box<Self>,
        /// If the flag is not set, use this.
        r#else: Box<Self>
    },
    /// Gets a string with `source`, modifies it with `modification`, and returns the result.
    /// # Errors
    /// If the call to [`StringModification::apply`] errors, returns that error.
    #[cfg(feature = "string-modification")]
    Modified {
        /// The source to get the string from.
        source: Box<Self>,
        /// The modification to apply to the string.
        modification: StringModification
    },
    /// Joins a list of strings.
    /// By default, `join` is `""` so the strings are concatenated.
    Join {
        /// The list of string sources to join.
        sources: Vec<Self>,
        /// The value to join `sources` with.
        #[serde(default)]
        join: String
    },
    /// Sends an HTTP GET request to the URL being cleaned and returns the value of the specified response header.
    /// # Errors
    /// If the call to [`Params::http_client`] returns an error, that error is returned.
    /// If the call to [`reqwest::RequestBuilder::send`] returns an error, that error is returned.
    /// If the specified header isn't found, returns the error [`StringSourceError::HeaderNotFound`].
    /// If the call to [`reqwest::header::HeaderValue::to_str`] returns an error, that error is returned.
    /// Note that, as I write this, [`reqwest::header::HeaderValue::to_str`] only works if the result is valid ASCII.
    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    ResponseHeader {
        /// The name of the response header to get the value of.
        name: String,
        /// The headers to send in the HTTP GET request.
        #[serde(default, with = "crate::glue::headermap")]
        headers: HeaderMap
    },
    /// Parses `source` as a URL and gets the specified value.
    /// Useful when used with [`Self::ResponseHeader`].
    ExtractPart {
        /// The string to parse and extract `part` from.
        source: Box<Self>,
        /// The part to extract from `source`.
        part: UrlPart
    },
    /// Sends an HTTP GET request to the URL being cleaned and extracts a string from the response's body.
    /// # Errors
    /// If the call to [`Params::http_client`] returns an error, that error is returned.
    /// If the call to [`reqwest::RequestBuilder::send`] returns an error, that error is returned.
    /// If the call to [`reqwest::Response::text`] returns an error, that error is returned.
    #[cfg(all(feature = "http", feature = "regex", not(target_family = "wasm")))]
    ExtractFromPage {
        /// The headers to send in the HTTP GET request.
        #[serde(default, with = "crate::glue::headermap")]
        headers: HeaderMap,
        /// The regex to use to extract part of the response body.
        regex: RegexWrapper,
        /// The substitution for use in [`regex::Captures::expand`].
        /// Defaults to `"$1"`.
        #[serde(default = "box_efp_expand")]
        expand: Box<Self>
    },
    /// Sends an HTTP request and returns a string from the response determined by the specified [`ResponseHandler`].
    #[cfg(all(feature = "advanced-requests", not(target_family = "wasm")))]
    HttpRequest(Box<RequestConfig>),
    /// If the contained [`Self`] returns `None`, instead return `Some(Cow::Borrowed(""))`
    NoneToEmptyString(Box<Self>)
}

fn box_efp_expand() -> Box<StringSource> {Box::new(StringSource::String("$1".to_string()))}

impl FromStr for StringSource {
    type Err = Infallible;

    /// Simply encase the provided string in a [`StringSource::String`] since it's the most common variant.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::String(s.to_string()))
    }
}

string_or_struct_magic!(StringSource);

/// The enum of all possible errors [`StringSource::get`] can return.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Error)]
pub enum StringSourceError {
    /// Returned when [`StringSource::Error`] is used.
    #[error("StringSource::Error was used.")]
    ExplicitError,
    /// Returned when a [`StringModificationError`] is encountered.
    #[cfg(feature = "string-modification")]
    #[error(transparent)]
    StringModificationError(#[from] StringModificationError),
    /// Returned when [`reqwest::Error`] is encountered.
    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    /// Returned when a requested HTTP response header is not found.
    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    #[error("The HTTP request response did not contain the requested header.")]
    HeaderNotFound,
    /// Returned when a [`reqwest::header::ToStrError`] is encountered.
    #[cfg(all(feature = "http", not(target_family = "wasm")))]
    #[error(transparent)]
    HeaderToStrError(#[from] reqwest::header::ToStrError),
    /// Returned when a [`url::ParseError`] is encountered.
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    /// Returned when a regex does not find any matches.
    #[error("A regex pattern did not find any matches.")]
    #[cfg(feature = "regex")]
    NoRegexMatchesFound,
    /// Returned when a call to [`StringSource::get`] returns `None` where it has to be `Some`.
    #[error("The specified StringSource returned None where it had to be Some.")]
    StringSourceIsNone,
    /// Returned when a [`RequestConfigError`] is encountered.
    #[cfg(all(feature = "advanced-requests", not(target_family = "wasm")))]
    #[error(transparent)]
    RequestConfigError(#[from] RequestConfigError),
    /// Returned when a [`ResponseHandlerError`] is encountered.
    #[cfg(all(feature = "advanced-requests", not(target_family = "wasm")))]
    #[error(transparent)]
    ReponseHandlerError(#[from] ResponseHandlerError)
}

impl StringSource {
    /// Gets the string from the source.
    /// # Errors
    /// See the documentation for [`Self`]'s variants for details.
    pub fn get<'a>(&'a self, url: &'a Url, params: &'a Params) -> Result<Option<Cow<'a, str>>, StringSourceError> {
        #[cfg(feature = "debug")]
        println!("Source: {self:?}");
        Ok(match self {
            Self::String(x) => Some(Cow::Borrowed(x.as_str())),
            Self::Part(x) => x.get(url),
            Self::Var(x) => params.vars.get(x).map(|x| Cow::Borrowed(x.as_str())),
            Self::IfFlag {flag, then, r#else} => if params.flags.contains(flag) {then} else {r#else}.get(url, params)?,
            #[cfg(feature = "string-modification")]
            Self::Modified {source, modification} => {
                match source.as_ref().get(url, params)? {
                    Some(x) => {
                        let mut x = x.into_owned();
                        modification.apply(&mut x, params)?;
                        Some(Cow::Owned(x))
                    },
                    None => None
                }
            },
            Self::Join {sources, join} => sources.iter().map(|source| source.get(url, params)).collect::<Result<Option<Vec<_>>, _>>()?.map(|x| Cow::Owned(x.join(join))),
            #[cfg(all(feature = "http", not(target_family = "wasm")))]
            Self::ResponseHeader{name, headers} => Some(Cow::Owned(params.http_client()?.get(url.as_str()).headers(headers.clone()).send()?.headers().get(name).ok_or(StringSourceError::HeaderNotFound)?.to_str()?.to_string())),
            Self::ExtractPart{source, part} => source.get(url, params)?.map(|x| Url::parse(&x)).transpose()?.and_then(|x| part.get(&x).map(|x| Cow::Owned(x.into_owned()))),
            #[cfg(all(feature = "http", feature = "regex", not(target_family = "wasm")))]
            Self::ExtractFromPage{headers, regex, expand} => if let Some(expand) = expand.get(url, params)? {
                let mut ret=String::new();
                regex.captures(&params.http_client()?.get(url.as_str()).headers(headers.clone()).send()?.text()?).ok_or(StringSourceError::NoRegexMatchesFound)?.expand(&expand, &mut ret);
                Some(Cow::Owned(ret))
            } else {
                Err(StringSourceError::StringSourceIsNone)?
            },
            #[cfg(all(feature = "advanced-requests", not(target_family = "wasm")))]
            Self::HttpRequest(config) => Some(Cow::Owned(config.response(url, params)?)),
            Self::NoneToEmptyString(source) => source.get(url, params)?.or(Some(Cow::Borrowed(""))),
            Self::Debug(source) => {
                let ret=source.get(url, params);
                eprintln!("=== StringSource::Debug ===\nSource: {source:?}\nURL: {url:?}\nParams: {params:?}\nret: {ret:?}");
                ret?
            },
            Self::Error => Err(StringSourceError::ExplicitError)?
        })
    }
}
