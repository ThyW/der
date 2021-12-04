use std::env;
use std::fmt;
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
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let formatted_string: String = match self {
            Self::Io(e) => {
                format!("Inner IO Error: {:?}", e.kind())
            }
            Self::EnvVar(e) => {
                format!("Environmental variable error: {:?}", e)
            }
            Self::Utf8Conversion(e) => {
                format!("Error converting to UTF-8: {}", e)
            }
            Self::Custom(e) => {
                format!("Error occured: {}", e)
            }
        };
        writeln!(
            fmt,
            "[ERROR] {}",
            formatted_string
        )
    }
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

impl From<String> for Error {
    fn from(other: String) -> Self {
        Self::Custom(other)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(other: string::FromUtf8Error) -> Self {
        Self::Utf8Conversion(other)
    }
}
