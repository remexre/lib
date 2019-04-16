//! Stuff for error-handling.

use antidote::Mutex;
#[cfg(feature = "backtrace")]
use backtrace::Backtrace;
use std::{
    error::Error,
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    mem::replace,
};

/// A convenient alias for `Result`.
pub type Result<T, E = Box<dyn Error + Send + Sync + 'static>> = ::std::result::Result<T, E>;

/// A wrapper for giving `ErrorKind`s causes and backtraces.
#[derive(Debug, Display)]
#[display(fmt = "{}", "kind")]
pub struct GenericError<T: Debug + Display> {
    kind: T,

    cause: Option<Box<dyn Error + 'static>>,

    // TODO: Don't derive Display if there's a backtrace.
    #[cfg(feature = "backtrace")]
    backtrace: Backtrace,
}

impl<T: Debug + Display> GenericError<T> {
    /// Returns the backtrace when the error was created.
    #[cfg(feature = "backtrace")]
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }

    /// Returns the kind of the error.
    pub fn kind(&self) -> &T {
        &self.kind
    }

    /// Creates an error with the given kind and cause.
    pub fn with_cause<C>(kind: T, cause: C) -> GenericError<T>
    where
        C: Into<Box<dyn Error + 'static>>,
    {
        GenericError {
            kind,
            cause: Some(cause.into()),
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace::new(),
        }
    }
}

impl<T: Debug + Display> Error for GenericError<T> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_ref().map(|boxed| &**boxed)
    }
}

impl<T: Debug + Display> From<T> for GenericError<T> {
    fn from(kind: T) -> GenericError<T> {
        GenericError {
            kind,
            cause: None,
            #[cfg(feature = "backtrace")]
            backtrace: Backtrace::new(),
        }
    }
}

/// An iterator over the causes of an error.
#[derive(Debug)]
pub struct ErrorCauseIter<'a>(Option<&'a (dyn Error + 'static)>);

impl<'a> From<&'a (dyn Error + 'static)> for ErrorCauseIter<'a> {
    fn from(err: &'a (dyn Error + 'static)) -> ErrorCauseIter<'a> {
        ErrorCauseIter(Some(err))
    }
}

impl<'a> Iterator for ErrorCauseIter<'a> {
    type Item = &'a dyn Error;

    fn next(&mut self) -> Option<&'a dyn Error> {
        let err = self.0?;
        self.0 = err.source();
        Some(err)
    }
}

// All the commented stuff will work on rustc 1.35.0+.

trait EFunc: Send {
    fn make_string(self: Box<Self>) -> String;
}

impl<F: FnOnce() -> String + Send> EFunc for F {
    fn make_string(self: Box<Self>) -> String {
        self()
    }
}

/// An error that is a wrapper around a `Formatter`.
pub struct E(Mutex<Result<String, Box<dyn EFunc>>>);
// pub struct E(Mutex<Result<String, Box<dyn FnOnce() -> String>>>);

impl E {
    fn as_string(&self) -> String {
        let mut lock = self.0.lock();
        match replace(&mut *lock, Ok("[panicked]".to_string())) {
            Ok(_) => {}
            // Err(func) => *lock = Ok(func()),
            Err(func) => *lock = Ok(func.make_string()),
        }
        match lock.as_ref() {
            Ok(s) => s.clone(),
            Err(_) => unreachable!(),
        }
    }

    #[doc(hidden)]
    pub fn from_closure<F: 'static + FnOnce() -> String + Send>(f: F) -> E {
        E(Mutex::new(Err(Box::new(f))))
    }
}

impl Debug for E {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str(&self.as_string())
    }
}

impl Display for E {
    fn fmt(&self, fmt: &mut Formatter) -> FmtResult {
        fmt.write_str(&self.as_string())
    }
}

impl Error for E {}

/// Creates an instance of `E` as a `Box<dyn Error>`.
///
/// ```rust
/// # use libremexre::err;
/// # use std::error::Error;
/// let e: Box<dyn Error> = err!("foo {} bar", 1);
/// assert_eq!(e.to_string(), "foo 1 bar");
/// ```
#[macro_export]
macro_rules! err {
    ($($tt:tt)*) => {{
        let e = $crate::errors::E::from_closure(move || std::format!($($tt)*));
        let e: std::boxed::Box<dyn std::error::Error + std::marker::Send + std::marker::Sync> =
            std::boxed::Box::new(e);
        e
    }};
}

/// Logs an error, including its causes.
#[cfg(feature = "log")]
pub fn log_err(err: &(dyn Error + 'static)) {
    use log::error;

    // Count the number of errors.
    let num_errs = ErrorCauseIter::from(err).count();

    if num_errs <= 1 {
        error!("{}", err);
    } else {
        let mut first = true;
        for err in ErrorCauseIter::from(err) {
            if first {
                first = false;
                error!("           {}", err);
            } else {
                error!("caused by: {}", err);
            }
        }
    }
}
