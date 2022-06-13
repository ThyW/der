use std::env;
use std::fmt;
use std::io;
use std::string;

/// Custom wrapper for `Result`.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// All the possible error variants.
#[derive(Debug)]
pub enum Error {
    /// Standard IO error.
    Io(io::Error),
    /// Environmental variable error.
    EnvVar(env::VarError),
    /// Conversion to UTF-8 string error.
    Utf8Conversion(string::FromUtf8Error),
    /// Error with a custom message.
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
        writeln!(fmt, "[\x1b[31mERROR\x1b[0m] {}", formatted_string)
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
