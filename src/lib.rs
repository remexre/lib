//! A bunch of utlities I found myself needing over and over again.
#![deny(
    bad_style,
    bare_trait_objects,
    const_err,
    dead_code,
    improper_ctypes,
    legacy_directory_ownership,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    plugin_as_library,
    private_in_public,
    safe_extern_statics,
    trivial_casts,
    trivial_numeric_casts,
    unconditional_recursion,
    unions_with_drop_fields,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_extern_crates,
    unused_import_braces,
    unused_parens,
    unused_qualifications,
    unused_results,
    while_true
)]

#[macro_use]
extern crate derive_more;

#[macro_use]
mod macros;

#[cfg(feature = "log")]
#[doc(hidden)]
pub extern crate log;
#[cfg(feature = "packer")]
#[doc(hidden)]
pub extern crate packer;
#[cfg(feature = "warp")]
#[doc(hidden)]
pub extern crate warp as warp_;

pub mod errors;
#[cfg(feature = "futures")]
pub mod futures;
#[cfg(all(feature = "log", feature = "pretty_env_logger"))]
mod logger;
#[cfg(all(feature = "futures", feature = "warp"))]
pub mod warp;

#[cfg(all(feature = "log", feature = "pretty_env_logger"))]
pub use logger::init_logger;
use std::sync::Arc;

/// Runs the given closure immediately. Mostly for use as replacement for `catch` blocks, which
/// seem to be taking a while to stabilize...
pub fn catch<F: FnOnce() -> T, T>(func: F) -> T {
    func()
}

/// Unwraps an `Arc`, cloning the inner value if necessary.
pub fn unwrap_arc<T: Clone>(arc: Arc<T>) -> T {
    Arc::try_unwrap(arc).unwrap_or_else(|arc| (*arc).clone())
}
