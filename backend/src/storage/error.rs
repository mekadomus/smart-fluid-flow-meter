use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorCode {
    DuplicateError,
    NotFoundError,
    RateLimitError,
    UndefinedError,
}

#[derive(Clone, Debug)]
pub struct Error {
    pub code: ErrorCode,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.code)
    }
}

impl std::error::Error for Error {}

pub fn not_found<T>() -> Result<T, Error> {
    Err(Error {
        code: ErrorCode::NotFoundError,
    })
}

pub fn undefined<T>() -> Result<T, Error> {
    Err(Error {
        code: ErrorCode::UndefinedError,
    })
}

pub fn rate_limit<T>() -> Result<T, Error> {
    Err(Error {
        code: ErrorCode::RateLimitError,
    })
}
