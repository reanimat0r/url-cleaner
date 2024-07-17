//! Various types to make URL Cleaner far more powerful.

use std::collections::HashMap;
use std::sync::OnceLock;

use thiserror::Error;
use url::Url;

mod url_part;
pub use url_part::*;
mod config;
pub use config::*;
mod tests;
pub use tests::*;
mod rules;
pub use rules::*;
mod string_location;
pub use string_location::*;
mod string_modification;
pub use string_modification::*;
mod string_source;
pub use string_source::*;
mod string_matcher;
pub use string_matcher::*;
#[cfg(all(feature = "advanced-requests", not(target_family = "wasm")))] mod advanced_requests;
#[cfg(all(feature = "advanced-requests", not(target_family = "wasm")))] pub use advanced_requests::*;
mod jobs;
pub use jobs::*;

/// An enum that transitively contains any possible error that can happen when cleaning a URL.
#[derive(Debug, Error)]
pub enum CleaningError {
    /// Returned when a [`GetConfigError`] os encountered.
    #[error(transparent)]
    GetConfigError(#[from] GetConfigError),
    /// Returned when a [`RuleError`] is encountered.
    #[error(transparent)]
    RuleError(#[from] RuleError),
    /// Returned when a [`url::ParseError`] is encountered.
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    /// Returned when a [`serde_json::Error`] is encountered.
    #[error(transparent)]
    SerdeJsonError(#[from] serde_json::Error)
}
