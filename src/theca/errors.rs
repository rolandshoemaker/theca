//  _   _
// | |_| |__   ___  ___ __ _
// | __| '_ \ / _ \/ __/ _` |
// | |_| | | |  __/ (_| (_| |
//  \__|_| |_|\___|\___\__,_|
//
// licensed under the MIT license <http://opensource.org/licenses/MIT>
//
// errors.rs
//   definitions for Error, a catch-all for converting various
//   lib errors.

use std::fmt;
use std::convert::From;
use std::error::Error as StdError;
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use std::time::SystemTimeError;
use time::ParseError;
use crypto::symmetriccipher::SymmetricCipherError;
use rustc_serialize::json::EncoderError;
use docopt;
use term;

pub type Result<T> = ::std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ErrorKind {
    Term(term::Error),
    InternalIo(IoError),
    Generic,
}

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub desc: String,
    pub detail: Option<String>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", &self.desc)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.desc
    }

    fn cause(&self) -> Option<&StdError> {
        match self.kind {
            ErrorKind::Term(ref e) => Some(e),
            ErrorKind::InternalIo(ref e) => Some(e),
            _ => None,
        }
    }
}

macro_rules! specific_fail {
    ($short:expr) => {{
        use errors::ErrorKind;
        Err(::std::convert::From::from(
            Error {
                kind: ErrorKind::Generic,
                desc: $short,
                detail: None
            }
        ))
    }}
}

macro_rules! specific_fail_str {
    ($s:expr) => {
        specific_fail!($s.to_string())
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

impl From<EncoderError> for Error {
    fn from(err: EncoderError) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: err.description().to_string(),
            detail: None,
        }
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: err.description().into(),
            detail: None,
        }
    }
}

impl From<term::Error> for Error {
    fn from(err: term::Error) -> Error {
        Error {
            desc: err.description().into(),
            kind: ErrorKind::Term(err),
            detail: None,
        }
    }
}

impl From<(ErrorKind, &'static str)> for Error {
    fn from((kind, desc): (ErrorKind, &'static str)) -> Error {
        Error {
            kind: kind,
            desc: desc.to_string(),
            detail: None,
        }
    }
}

impl From<SystemTimeError> for Error {
    fn from(err: SystemTimeError) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: err.description().into(),
            detail: None,
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: format!("time parsing error: {}", err),
            detail: None,
        }
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: format!("is this profile encrypted? ({})", err),
            detail: None,
        }
    }
}

impl From<SymmetricCipherError> for Error {
    fn from(_: SymmetricCipherError) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: "invalid encryption key".to_string(),
            detail: None,
        }
    }
}

impl From<docopt::Error> for Error {
    fn from(err: docopt::Error) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: err.to_string(),
            detail: None,
        }
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Error {
        Error {
            kind: ErrorKind::Generic,
            desc: "formatting error".to_string(),
            detail: None,
        }
    }
}
