use std::env;
use std::io;
use std::string;

/// Custom wrapper for `Result`.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// All the possible errors.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    EnvVar(env::VarError),
    Utf8Conversion(string::FromUtf8Error),
    Custom(&'static str),
}

impl From<io::Error> for Error {
    fn from(other: io::Error) -> Self {
        Self::Io(other)
    }
}

impl From<env::VarError> for Error {
    fn from(other: env::VarError) -> Self {
        Self::EnvVar(other)
    }
}

impl From<&'static str> for Error {
    fn from(other: &'static str) -> Self {
        Self::Custom(other)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(other: string::FromUtf8Error) -> Self {
        Self::Utf8Conversion(other)
    }
}
