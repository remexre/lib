//! Utilities for use with [warp](https://github.com/seanmonstar/warp).

use futures::{Async, Future};
use std::error::Error;
use warp::Rejection;

/// A `Filter` that serves static files from the given directory.
///
/// ```
/// # use libremexre::packer_statics;
/// let filter = packer_statics!("src");
/// let req = warp::test::request()
///     .path("/lib.rs")
///     .filter(&filter)
///     .unwrap();
///
/// let got: &[u8] = req.body();
/// let exp: &[u8] = include_bytes!("../lib.rs");
/// assert_eq!(got, exp);
/// ```
#[cfg(feature = "packer")]
#[macro_export]
macro_rules! packer_statics {
    ($dir:literal) => {{
        use $crate::{
            log::warn,
            packer::Packer,
            warp_::{http::{header::CONTENT_TYPE, Response}, reject::custom, Filter},
        };
        use std::path::Path;

        #[derive(Packer)]
        #[folder = $dir]
        struct Assets;

        $crate::warp_::path::tail().and_then(|path: $crate::warp_::path::Tail| {
            let path = path.as_str();
            Assets::get(path)
                .ok_or_else($crate::warp_::reject::not_found)
                .and_then(|body| {
                    let ext = $crate::coerce!(path.as_ref() => &Path)
                        .extension()
                        .and_then(|s| s.to_str());
                    let ct = match ext {
                        Some("css") => "text/css; charset=utf-8",
                        Some("html") => "text/html; charset=utf-8",
                        Some("js") => "application/javascript",
                        Some("txt") => "text/plain; charset=utf-8",
                        Some("woff2") => "font/woff2",
                        _ => {
                            warn!("Unknown extension for static file: {:?}", ext);
                            warn!("File a bug against libremexre; thanks!");
                            "application/octet-stream"
                        }
                    };
                    Response::builder()
                        .header(CONTENT_TYPE, ct)
                        .body(body)
                        .map_err(custom)
                })
        })
    }};
}

/// The type of a responder. Since `impl Trait` can't be used in `type` items, this magics one up.
#[macro_export]
macro_rules! Resp {
    () => { $crate::warp_::filters::BoxedFilter<(impl $crate::warp_::Reply,)> };
}

/// Inserts `.or(...)` between the given filters.
#[macro_export]
macro_rules! route_any {
    ($hm:ident $hp:tt => $h:expr $(, $tm:ident $tp:tt => $t:expr)* $(,)*) => {{
        use $crate::warp_::Filter;
        route_any!(@internal @path $hm $hp).and($h)
            $(.or(route_any!(@internal @path $tm $tp).and($t)))*
    }};

    (@internal @path GET ()) => {{ warp::get2() }};
    (@internal @path POST ()) => {{ warp::post2() }};
    (@internal @path $m:ident $p:tt) => {{
        use warp::path;
        path! $p.and(route_any!(@internal @path $m ()))
    }};
}

/// An extension trait for Futures.
pub trait FutureExt: Sized {
    /// Converts an error to a `warp::Rejection`.
    fn err_to_rejection(self) -> ErrToRejection<Self>;
}

impl<F> FutureExt for F
where
    F: Future,
    F::Error: 'static + Error + Send + Sync,
{
    fn err_to_rejection(self) -> ErrToRejection<Self> {
        ErrToRejection(self)
    }
}

/// A wrapper that converts errors to Rejections.
#[derive(Debug)]
#[doc(hidden)]
pub struct ErrToRejection<F>(F);

impl<F> Future for ErrToRejection<F>
where
    F: Future,
    F::Error: 'static + Error + Send + Sync,
{
    type Item = F::Item;
    type Error = Rejection;

    fn poll(&mut self) -> Result<Async<F::Item>, Rejection> {
        match self.0.poll() {
            Ok(x) => Ok(x),
            Err(e) => Err(warp::reject::custom(e)),
        }
    }
}
