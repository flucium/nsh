use std::fmt;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum ErrorKind {
    WrongSyntax,
}

#[derive(Debug, Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self {
            kind: kind,
            message: message,
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
    
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}
