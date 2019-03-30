use std::fmt;

#[derive(Debug)]
pub enum ErrorKind {
    /// incorrect action variant is used to try to convert
    /// into action argument
    IncorrectAction,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorKind::IncorrectAction => write!(f, "Incorrect action when converting (bug)"),
        }
    }
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

impl std::error::Error for Error {}
