use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    /// incorrect action variant is used to try to convert
    /// into action argument
    IncorrectAction,
    Timeout,
    /// other error source
    Source,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::IncorrectAction => write!(f, "Incorrect action when converting (bug)"),
            ErrorKind::Timeout => write!(f, "Timeout"),
            ErrorKind::Source => write!(f, "Error from another source"),
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
}
