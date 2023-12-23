#[cfg(feature = "glob")]
mod enabled;
#[cfg(not(feature = "glob"))]
mod disabled;

#[cfg(feature = "glob")]
pub use enabled::GlobWrapper;
#[cfg(not(feature = "glob"))]
pub use disabled::GlobWrapper;
