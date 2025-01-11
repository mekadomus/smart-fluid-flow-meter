#[derive(Debug)]
pub enum ErrorCode {
    UndefinedError,
    DuplicateError,
}

#[derive(Debug)]
pub struct Error {
    pub code: ErrorCode,
}
