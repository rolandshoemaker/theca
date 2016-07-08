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

use std::fmt;
use std::convert::From;
use std::error::Error;
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use std::time::SystemTimeError;
use time::ParseError;
use crypto::symmetriccipher::SymmetricCipherError;
use rustc_serialize::json::EncoderError;
use docopt;
use term;

pub use self::ErrorKind::{TermError, InternalIoError, GenericError};

#[derive(Debug)]
pub enum ErrorKind {
    TermError(term::Error),
    InternalIoError(IoError),
    GenericError,
}

#[derive(Debug)]
pub struct ThecaError {
    pub kind: ErrorKind,
    pub desc: String,
    pub detail: Option<String>,
}

impl fmt::Display for ThecaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.desc)
    }
}

impl Error for ThecaError {
    fn description(&self) -> &str {
        &self.desc
    }

    fn cause(&self) -> Option<&Error> {
        match self.kind {
            ErrorKind::TermError(ref e) => Some(e),
            ErrorKind::InternalIoError(ref e) => Some(e),
            _ => None,
        }
    }
}

macro_rules! specific_fail {
    ($short:expr) => {
        return Err(::std::convert::From::from(
            ThecaError {
                kind: GenericError,
                desc: $short,
                detail: None
            }
        ))
    }
}

macro_rules! specific_fail_str {
    ($s:expr) => {
        return specific_fail!($s.to_string())
    }
}

macro_rules! try_errno {
    ($e:expr) => {
        {
            if $e != 0 {
                return Err(
                    ::std::convert::From::from(
                        IoError::last_os_error()
                    )
                );
            }
        }
    }
}

impl From<EncoderError> for ThecaError {
    fn from(err: EncoderError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.description().to_string(),
            detail: None,
        }
    }
}

impl From<IoError> for ThecaError {
    fn from(err: IoError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.description().into(),
            detail: None,
        }
    }
}

impl From<term::Error> for ThecaError {
    fn from(err: term::Error) -> ThecaError {
        ThecaError {
            desc: err.description().into(),
            kind: TermError(err),
            detail: None,
        }
    }
}

impl From<(ErrorKind, &'static str)> for ThecaError {
    fn from((kind, desc): (ErrorKind, &'static str)) -> ThecaError {
        ThecaError {
            kind: kind,
            desc: desc.to_string(),
            detail: None,
        }
    }
}

impl From<SystemTimeError> for ThecaError {
    fn from(err: SystemTimeError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.description().into(),
            detail: None,
        }
    }
}

impl From<ParseError> for ThecaError {
    fn from(err: ParseError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: format!("time parsing error: {}", err),
            detail: None,
        }
    }
}

impl From<FromUtf8Error> for ThecaError {
    fn from(err: FromUtf8Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: format!("is this profile encrypted? ({})", err),
            detail: None,
        }
    }
}

impl From<SymmetricCipherError> for ThecaError {
    fn from(_: SymmetricCipherError) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: "invalid encryption key".to_string(),
            detail: None,
        }
    }
}

impl From<docopt::Error> for ThecaError {
    fn from(err: docopt::Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: err.to_string(),
            detail: None,
        }
    }
}

impl From<fmt::Error> for ThecaError {
    fn from(_: fmt::Error) -> ThecaError {
        ThecaError {
            kind: GenericError,
            desc: "formatting error".to_string(),
            detail: None,
        }
    }
}
