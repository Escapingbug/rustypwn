use std::fmt;
use subprocess::PopenError;
use subprocess::ExitStatus;

#[derive(Debug)]
pub enum ErrorKind {
    /// incorrect action variant is used to try to convert
    /// into action argument
    IncorrectAction,
    /// simple timeout
    Timeout,
    /// process open error
    Popen,
    /// get terminates when trying to send or recv
    UnexpectedTerminate(ExitStatus),
    /// other error source
    Source,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::IncorrectAction => write!(f, "Incorrect action when converting (bug)"),
            ErrorKind::Timeout => write!(f, "Timeout"),
            ErrorKind::Source => write!(f, "Error from another source"),
            ErrorKind::Popen => write!(f, "Process open error"),
            ErrorKind::UnexpectedTerminate(status) => {
                let _ = write!(f, "process terminates ");
                match status {
                    ExitStatus::Exited(res) => write!(f, "with exit code {}", res),
                    ExitStatus::Signaled(res) => write!(f, "because of signal {}", res),
                    ExitStatus::Other(res) => write!(f, "because of unknown reason {}", res),
                    ExitStatus::Undetermined => write!(f, "because of unknown reason"),
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub source: Option<Box<dyn std::error::Error + 'static>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res = self.kind.fmt(f);
        match &self.source {
            Some(source) => source.fmt(f),
            _ => res,
        }
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn from_kind(kind: ErrorKind) -> Self {
        Error {
            kind: kind,
            source: None,
        }
    }

    pub fn from_source(source: Box<dyn std::error::Error + 'static>) -> Self {
        Error {
            kind: ErrorKind::Source,
            source: Some(source)
        }
    }

    /// Since we have too many timeouts, make it a helper
    pub fn timeout() -> Self {
        Error {
            kind: ErrorKind::Timeout,
            source: None,
        }
    }
}

macro_rules! impl_from_source {
    ($source_err:path) => {
        impl From<$source_err> for Error {
            fn from(source: $source_err) -> Self {
                Self::from_source(Box::new(source))
            }
        }
    }
}

impl_from_source!(PopenError);
impl_from_source!(std::io::Error);
