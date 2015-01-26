//  _   _                    
// | |_| |__   ___  ___ __ _ 
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// licensed under the MIT license <http://opensource.org/licenses/MIT>
//
// errors.rs
//   definitions for ThecaError, a catch-all for converting various 
//   lib errors.

use core::fmt;
use core::error::Error;
use docopt;
use time::{ParseError};
use std::error::{FromError};
use crypto::symmetriccipher::SymmetricCipherError;
use std::io::{IoError};
use std::string::FromUtf8Error;
use rustc_serialize::json::EncoderError;

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
            let err = $e;
            if err != 0 {
                return Err(::std::error::FromError::from_error(IoError::from_errno(errno() as usize, true)));
            }
        }
    }
}

impl FromError<EncoderError> for ThecaError {
    fn from_error(err: EncoderError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.description().to_string(),
            detail: None
        }
    }
}

impl FromError<IoError> for ThecaError {
    fn from_error(err: IoError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.desc.to_string(),
            detail: err.detail
        }
    }
}

impl FromError<(ErrorKind, &'static str)> for ThecaError {
    fn from_error((kind, desc): (ErrorKind, &'static str)) -> ThecaError {
        ThecaError { kind: kind, desc: desc.to_string(), detail: None }
    }
}

impl FromError<ParseError> for ThecaError {
    fn from_error(err: ParseError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: format!("\ntime parsing error: {}", err),
            detail: None
        }
    }
}

impl FromError<FromUtf8Error> for ThecaError {
    fn from_error(err: FromUtf8Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: format!("\nerror parsing utf-8, is profile encrypted?\n({})", err),
            detail: None
        }
    }
}

impl FromError<SymmetricCipherError> for ThecaError {
    fn from_error(_: SymmetricCipherError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: "\ninvalid encryption key".to_string(),
            detail: None
        }
    }
}

impl FromError<docopt::Error> for ThecaError {
    fn from_error(err: docopt::Error) -> ThecaError {
        ThecaError { kind: GenericError, desc: err.to_string(), detail: None }
    }
}

impl FromError<fmt::Error> for ThecaError {
    fn from_error(_: fmt::Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: "\nformatting error".to_string(),
            detail: None
        }
    }
}
