use core;
use docopt;
use time;
use std::error;
use crypto::symmetriccipher::SymmetricCipherError;
use std::io::{IoError};
use std::string::FromUtf8Error;

pub use self::ErrorKind::{
    InternalIoError,
    GenericError
};

pub enum ErrorKind {
    InternalIoError(IoError),
    GenericError
}

pub struct ThecaError {
    pub kind: ErrorKind,
    pub desc: String,
    pub detail: Option<String>
}

#[macro_export]
macro_rules! specific_fail {
    ($short:expr) => {
        return Err(::std::error::FromError::from_error(
            ThecaError {
                kind: GenericError,
                desc: $short,
                detail: None
            }
        ))
    }
}

macro_rules! try_errno {
    ($e:expr) => {
        {
            use std::io::{IoError};
            use std::os::errno;
            let err = $e;
            if err != 0 {
                return Err(::std::error::FromError::from_error(IoError::from_errno(errno() as usize, true)));
            }
        }
    }
}

impl error::FromError<IoError> for ThecaError {
    fn from_error(err: IoError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.desc.to_string(),
            detail: None
        }
    }
}

impl error::FromError<(ErrorKind, &'static str)> for ThecaError {
    fn from_error((kind, desc): (ErrorKind, &'static str)) -> ThecaError {
        ThecaError { kind: kind, desc: desc.to_string(), detail: None }
    }
}

impl error::FromError<time::ParseError> for ThecaError {
    fn from_error(err: time::ParseError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: format!("time parsing error: {}.", err),
            detail: None
        }
    }
}

impl error::FromError<FromUtf8Error> for ThecaError {
    fn from_error(err: FromUtf8Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: format!("error parsing utf-8, is profile encrypted?\n({})", err),
            detail: None
        }
    }
}

impl error::FromError<SymmetricCipherError> for ThecaError {
    fn from_error(_: SymmetricCipherError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: "invalid encryption key".to_string(),
            detail: None
        }
    }
}

impl error::FromError<docopt::Error> for ThecaError {
    fn from_error(err: docopt::Error) -> ThecaError {
        ThecaError { kind: GenericError, desc: err.to_string(), detail: None }
    }
}

impl error::FromError<core::fmt::Error> for ThecaError {
    fn from_error(_: core::fmt::Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: "formatting error.".to_string(),
            detail: None
        }
    }
}
