//! Utilities for use with [futures](https://docs.rs/futures/0.1.25/futures/) and
//! [tokio](https://docs.rs/tokio/0.1.15/tokio/).

use futures::{future::poll_fn, Future};

/// A higher-level version of `tokio_threadpool::blocking`.
#[cfg(all(feature = "tokio", feature = "tokio-threadpool"))]
pub fn blocking<E, F, T>(func: F) -> impl Future<Item = T, Error = E>
where
    F: FnOnce() -> Result<T, E>,
{
    let mut func = Some(func);
    poll_fn(move || {
        tokio_threadpool::blocking(|| (func.take().unwrap())())
            .map_err(|_| panic!("Blocking operations must be run inside a Tokio thread pool!"))
    })
    .and_then(|r| r)
}
