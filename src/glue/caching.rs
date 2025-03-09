//! Allows caching to an SQLite file.
//! 
//! Enabled by the `caching` feature flag.

use std::sync::{Arc, Mutex};
use std::str::FromStr;
use std::cell::OnceCell;
use std::path::Path;

use thiserror::Error;
use serde::{Serialize, Deserialize};
use diesel::prelude::*;

use crate::util::*;

#[allow(clippy::missing_docs_in_private_items, reason = "File is auto-generated by diesel's CLI.")]
mod schema;
pub use schema::cache;

/// The SQL command used to initialize a cache database.
pub const DB_INIT_COMMAND: &str = r#"CREATE TABLE cache (
    id INTEGER NOT NULL PRIMARY KEY,
    category TEXT NOT NULL,
    "key" TEXT NOT NULL,
    value TEXT
)"#;

/// An entry in the [`cache`] table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = cache)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CacheEntry {
    /// The ID of the entry.
    pub id: i32,
    /// The category of the entry.
    pub category: String,
    /// The key of the entry.
    pub key: String,
    /// The value of the entry.
    pub value: Option<String>
}

/// An addition to the [`cache`] table.
#[derive(Debug, PartialEq, Eq, Serialize, Insertable)]
#[diesel(table_name = cache)]
pub struct NewCacheEntry<'a> {
    /// The category of the new entry.
    pub category: &'a str,
    /// The key of the new entry.
    pub key: &'a str,
    /// The value of the new entry.
    pub value: Option<&'a str>
}

/// Convenience wrapper to contain the annoyingness of it all.
/// 
/// Internally it's an [`Arc`] of a [`Mutex`] so cloning is O(1) and sharing immutable references is not a problem.
#[derive(Debug, Clone, Default)]
pub struct Cache(pub Arc<Mutex<InnerCache>>);

/// The internals of [`Cache`] that handles lazily connecting.
#[derive(Default)]
pub struct InnerCache {
    /// The path being connected to.
    path: CachePath,
    /// The actual [`SqliteConnection`].
    connection: OnceCell<SqliteConnection>
}

impl PartialEq for InnerCache {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for InnerCache {}

/// Specifies where to store the cache.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub enum CachePath {
    /// Store the cache in RAM.
    #[default]
    Memory,
    /// Store the cache in a file.
    Path(String)
}

impl CachePath {
    /// Return the [`str`] this came from.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Memory => ":memory:",
            Self::Path(x) => x
        }
    }

    /// If [`Self::Path`], return the path.
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Memory => None,
            Self::Path(x) => Some(x.strip_prefix("file://").unwrap_or(x).as_ref())
        }
    }
}

impl AsRef<str> for CachePath {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for CachePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CachePath {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.to_string().into())
    }
}

impl From<&str> for CachePath {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for CachePath {
    fn from(value: String) -> Self {
        match &*value {
            ":memory:" => Self::Memory,
            _ => Self::Path(value)
        }
    }
}

crate::util::string_or_struct_magic!(CachePath);

impl ::core::fmt::Debug for InnerCache {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        f.debug_struct("InnerCache")
            .field("path", &self.path)
            .field("connection", if self.connection.get().is_some() {&"OnceCell(..)"} else {&"OnceCell(<uninit>)"})
            .finish()
    }
}

impl FromStr for Cache {
    type Err = <InnerCache as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        InnerCache::from_str(s).map(Into::into)
    }
}

impl<T: Into<InnerCache>> From<T> for Cache {
    fn from(value: T) -> Self {
        Self(Arc::new(Mutex::new(value.into())))
    }
}

impl FromStr for InnerCache {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<&str> for InnerCache {
    fn from(value: &str) -> Self {
        value.to_string().into()
    }
}

impl From<String> for InnerCache {
    fn from(value: String) -> Self {
        InnerCache { path: value.into(), connection: OnceCell::new() }
    }
}

impl From<CachePath> for InnerCache {
    fn from(value: CachePath) -> Self {
        InnerCache { path: value, connection: OnceCell::new() }
    }
}

/// The enum of errors [`Cache::read`] and [`InnerCache::read`] can return.
#[derive(Debug, Error)]
pub enum ReadFromCacheError {
    /// Returned when the inner [`Mutex`] is poisoned.
    #[error("{0}")]
    MutexPoisonError(String),
    /// Returned when a [`diesel::result::Error`] is encountered.
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),
    /// Returned when a [`ConnectCacheError`] is encountered.
    #[error(transparent)]
    ConnectCacheError(#[from] ConnectCacheError)
}

/// The enum of errors [`Cache::write`] and [`InnerCache::write`] can return.
#[derive(Debug, Error)]
pub enum WriteToCacheError {
    /// Returned when the inner [`Mutex`] is poisoned.
    #[error("{0}")]
    MutexPoisonError(String),
    /// Returned when a [`diesel::result::Error`] is encountered.
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error),
    /// Returned when a [`ConnectCacheError`] is encountered.
    #[error(transparent)]
    ConnectCacheError(#[from] ConnectCacheError)
}

impl Cache {
    /// Reads a string from the cache.
    /// # Errors
    /// If the call to [`Mutex::lock`] returns an error, that error is returned.
    /// 
    /// If the call to [`InnerCache::read`] returns an error, that error is returned.
    pub fn read(&self, category: &str, key: &str) -> Result<Option<Option<String>>, ReadFromCacheError> {
        self.0.lock().map_err(|e| ReadFromCacheError::MutexPoisonError(e.to_string()))?.read(category, key)
    }

    /// Writes a string to the cache.
    /// # Errors
    /// If the call to [`Mutex::lock`] returns an error, that error is returned.
    /// 
    /// If the call to [`InnerCache::write`] returns an error, that error is returned.
    pub fn write(&self, category: &str, key: &str, value: Option<&str>) -> Result<(), WriteToCacheError> {
        self.0.lock().map_err(|e| WriteToCacheError::MutexPoisonError(e.to_string()))?.write(category, key, value)
    }
}

/// The enum of errors [`InnerCache::connect`] can return.
#[derive(Debug, Error)]
pub enum ConnectCacheError {
    /// Returned when a [`diesel::ConnectionError`] is encountered.
    #[error(transparent)]
    ConnectionError(#[from] diesel::ConnectionError),
    /// Returned when a [`std::io::Error`] is encountered.
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    /// Returned when a [`diesel::result::Error`] is encountered.
    #[error(transparent)]
    DieselError(#[from] diesel::result::Error)
}

impl InnerCache {
    /// Returns the path being connected to.
    pub fn path(&self) -> &CachePath {
        &self.path
    }

    /// If connected, returns a mutable reference to the connection.
    pub fn connection(&mut self) -> Option<&mut SqliteConnection> {
        self.connection.get_mut()
    }

    /// If already connected, just returns the connection.
    /// 
    /// If unconnected, connect to the path then return the connection.
    /// 
    /// If the path is a file and doesn't exist, makes the file.
    /// 
    /// If the path is `:memory:`, the database is stored ephemerally in RAM and not saved to disk.
    /// # Errors
    /// If the call to [`std::fs::exists`] returns an error, that error is returned.
    /// 
    /// If the call to [`std::fs::File::create_new`] returns an error, that error is returned.
    /// 
    /// If initializing the database returns an error, that error is returned.
    /// 
    /// If the call to [`SqliteConnection::establish`] returns an error, that error is returned.
    #[allow(clippy::missing_panics_doc, reason = "Doesn't panic, but should be replaced with OnceCell::get_or_try_init once that's stable.")]
    pub fn connect(&mut self) -> Result<&mut SqliteConnection, ConnectCacheError> {
        debug!(InnerCache::connect, self);
        if self.connection.get().is_none() {
            let mut needs_init = self.path == CachePath::Memory;
            if let CachePath::Path(path) = &self.path {
                if !std::fs::exists(path)? {
                    needs_init = true;
                    std::fs::File::create_new(path)?;
                }
            }
            let mut connection = SqliteConnection::establish(self.path.as_str())?;
            if needs_init {
                diesel::sql_query(DB_INIT_COMMAND).execute(&mut connection)?;
            }
            self.connection.set(connection).map_err(|_| ()).expect("The connection to have just been confirmed unset.");
        }
        Ok(self.connection.get_mut().expect("The connection to have just been set."))
    }

    /// Disconnects and drops the contained [`SqliteConnection`].
    pub fn disconnect(&mut self) {
        let _ = self.connection.take();
    }

    /// Reads a string from the cache.
    /// 
    /// The outer [`Option`] says if there's a matching cache entry.
    /// 
    /// The inner [`Option`] is the cache entry.
    /// # Errors
    /// If the call to [`Self::connect`] returns an error, that error is returned.
    /// 
    /// If the call to [`RunQueryDsl::get_result`] returns an error, that error is returned.
    pub fn read(&mut self, category: &str, key: &str) -> Result<Option<Option<String>>, ReadFromCacheError> {
        debug!(InnerCache::read, self, category, key);
        Ok(cache::dsl::cache
            .filter(cache::dsl::category.eq(category))
            .filter(cache::dsl::key.eq(key))
            .limit(1)
            .select(CacheEntry::as_select())
            .load(self.connect()?)?
            .first()
            .map(|cache_entry| cache_entry.value.to_owned()))
    }

    /// Overwrites an entry to the cache.
    /// 
    /// If an entry doesn't exist, it is made.
    /// # Errors
    /// If the call to [`Self::connect`] returns an error, that error is returned.
    /// 
    /// If the call to [`RunQueryDsl::get_result`] returns an error, that error is returned.
    pub fn write(&mut self, category: &str, key: &str, value: Option<&str>) -> Result<(), WriteToCacheError> {
        debug!(InnerCache::write, self, category, key, value);
        diesel::replace_into(cache::table)
            .values(&NewCacheEntry {category, key, value})
            .returning(CacheEntry::as_returning())
            .get_result(self.connect()?)?;
        Ok(())
    }
}

impl From<InnerCache> for (CachePath, OnceCell<SqliteConnection>) {
    fn from(value: InnerCache) -> Self {
        (value.path, value.connection)
    }
}
