use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    InternalError(String),
    NotAuthenticatedYetError,
    ServerError(String),
    ChannelRecvErr(async_std::channel::RecvError),
    ParseError(url::ParseError),
    ParseIntError(std::num::ParseIntError),
    ParseUtf8Error(std::str::Utf8Error),
    FromUtf8Error(std::string::FromUtf8Error),
    IOError(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Error::InternalError(ref e) => e.fmt(f),
            Error::ServerError(ref e) => e.fmt(f),
            Error::ChannelRecvErr(ref e) => e.fmt(f),
            Error::NotAuthenticatedYetError => f.write_str("you should authenticate first"),
            Error::ParseError(ref e) => e.fmt(f),
            Error::ParseIntError(ref e) => e.fmt(f),
            Error::ParseUtf8Error(ref e) => e.fmt(f),
            Error::FromUtf8Error(ref e) => e.fmt(f),
            Error::IOError(ref e) => e.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match *self {
            Error::InternalError(ref _e) => None,
            Error::ServerError(ref _e) => None,
            Error::ChannelRecvErr(ref _e) => None,
            Error::NotAuthenticatedYetError => None,
            Error::ParseError(ref e) => Some(e),
            Error::ParseIntError(ref e) => Some(e),
            Error::ParseUtf8Error(ref e) => Some(e),
            Error::FromUtf8Error(ref e) => Some(e),
            Error::IOError(ref e) => Some(e),
        }
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Error {
        Error::ParseIntError(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Error {
        Error::ParseUtf8Error(err)
    }
}

impl From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Error {
        Error::FromUtf8Error(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<async_std::channel::RecvError> for Error {
    fn from(err: async_std::channel::RecvError) -> Error {
        Error::ChannelRecvErr(err)
    }
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Error {
        Error::ParseError(err)
    }
}
