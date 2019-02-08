//! Stuff for error-handling.

#[cfg(feature = "backtrace")]
use backtrace::Backtrace;
use std::{
    error::Error,
    fmt::{Debug, Display},
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
