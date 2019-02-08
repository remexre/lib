//! A bunch of utlities I found myself needing over and over again.

#[macro_use]
extern crate derive_more;

#[macro_use]
pub mod macros;

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
#[cfg(feature = "warp")]
pub mod warp;
