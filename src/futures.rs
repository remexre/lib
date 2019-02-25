//! Utilities for use with [futures](https://docs.rs/futures/0.1.25/futures/) and
//! [tokio](https://docs.rs/tokio/0.1.15/tokio/).

use futures::{future::poll_fn, Async, Future, Stream};
use std::{collections::HashMap, hash::Hash};

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

/// Allows selecting over several streams, keyed by identifiers. Polls in a round-robin fashion.
/// Streams are dropped when they yield `Ok(Ready(None))`.
pub struct SelectSet<K: Clone + Eq + Hash, S: Stream> {
    current: usize,
    keys: Vec<K>,
    streams: HashMap<K, S>,
}

impl<K: Clone + Eq + Hash, S: Stream> SelectSet<K, S> {
    /// Creates a new, empty SelectSet. Note that this will always return `Ok(NotReady)` when
    /// polled, until a stream is added!
    pub fn new() -> SelectSet<K, S> {
        SelectSet::default()
    }

    /// Adds a new stream with the given key. If a stream was already present, returns it.
    pub fn add(&mut self, key: K, stream: S) -> Option<S> {
        if let Some(prev) = self.streams.insert(key.clone(), stream) {
            Some(prev)
        } else {
            self.keys.push(key);
            None
        }
    }

    /// Removes a stream by key, if it exists.
    pub fn remove(&mut self, key: &K) -> Option<S> {
        self.streams.remove(key).map(|stream| {
            // This may deviate from round-robin behavior, when what we're removing was just
            // polled. However, the code to fix this is more trouble than it's worth.
            let n = self.keys.iter().position(|k| k == key).unwrap();
            self.keys.remove(n);

            stream
        })
    }
}

impl<K: Clone + Eq + Hash, S: Stream> Default for SelectSet<K, S> {
    fn default() -> SelectSet<K, S> {
        SelectSet {
            current: 0,
            keys: Vec::new(),
            streams: HashMap::new(),
        }
    }
}

impl<K: Clone + Eq + Hash, S: Stream> Stream for SelectSet<K, S> {
    type Item = S::Item;
    type Error = S::Error;

    fn poll(&mut self) -> Result<Async<Option<S::Item>>, S::Error> {
        if self.keys.is_empty() {
            return Ok(Async::NotReady);
        }

        self.current = (self.current + 1) % self.keys.len();
        let r = self
            .streams
            .get_mut(&self.keys[self.current])
            .unwrap()
            .poll();

        if let Ok(Async::Ready(None)) = r {
            let key = self.keys[self.current].clone();
            self.remove(&key);
        }
        r
    }
}
