//! URL Cleaner is an extremely powerful URL manipulation tool.
//! 
//! The built-in default config is focused on removing trackers and other maltext, expanding redirects, and a few bells and whistles for convenience.
//! # Examples
//! ```
//! use std::borrow::Cow;
//! use std::str::FromStr;
//! use url::Url;
//! 
//! use url_cleaner::types::*;
//! 
//! // See also the default config, which can be used with `Config::get_default().unwrap()`.
//! // Also when using serde to deserialize a `Config` from a file (the intended experience), you can omit all fields except for `rules`.
//! let config = Config {
//!     docs: Default::default(),
#![cfg_attr(feature = "cache", doc = "    cache_path: Default::default(),")]
//!     params: Default::default(),
//!     commons: Default::default(),
//!     rules: Rules(vec![
//!         Rule::Normal {
//!             condition: Condition::Always,
//!             mapper: Mapper::RemoveQueryParams(["utm_source".to_string()].into())
//!         }
//!     ])
//! };
//! 
//! let mut jobs = Jobs {
//!     jobs_config: JobsConfig {
//!         config: Cow::Borrowed(&config),
#![cfg_attr(feature = "cache", doc = "        // Doesn't do anything expensive until actually used.")]
#![cfg_attr(feature = "cache", doc = "        // You should use a global static `OnceLock` if you have to make multiple `Jobs`s with the same `Cache`.")]
#![cfg_attr(feature = "cache", doc = "        // That's fine because cloning a `Cache` is extremely cheap, because it's an `Arc<Mutex<InnerCache>>`.")]
#![cfg_attr(feature = "cache", doc = "        cache: config.cache_path.as_str().into()")]
//!     },
//!     context: Default::default(),
//!     // Ideally you'll be handling URLs in bulk.
//!     job_configs_source: Box::new([
//!         JobConfig::from_str("https://example.com?utm_source=url-cleaner-docs")
//!     ].into_iter())
//! };
//! 
//! for job in jobs.iter() {
//!     println!("{}", job.unwrap().r#do().unwrap());
//! }
//! ```

pub mod glue;
pub mod types;
pub mod testing;
pub(crate) mod util;

pub use types::{Config, Jobs, JobConfig};
