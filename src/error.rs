use std::io;
use std::env;

/// Custom wrapper for `Result`.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// All the possible errors.
#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    EnvVar(env::VarError),
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
