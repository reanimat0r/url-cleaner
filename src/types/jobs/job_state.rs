//! The state of a job as it's happening.

use url::Url;

use crate::types::*;
use crate::glue::*;

/// The current state of the job.
#[derive(Debug)]
pub struct JobState<'a> {
    /// The URL being modified.
    pub url: &'a mut Url,
    /// The context surrounding the URL.
    pub context: &'a UrlContext,
    /// The flags, variables, etc. defined by the job initiator.
    pub params: &'a Params,
    /// The string vars created and managed by the config.
    pub vars: HashMap<String, String>,
    /// The cache handler.
    #[cfg(feature = "cache")]
    pub cache_handler: &'a CacheHandler
}
