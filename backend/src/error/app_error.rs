use crate::json::extractor::Extractor;
use crate::storage::error::Error;

use axum::{
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::fmt;

pub enum AppErrorCode {
    InvalidInput,
    InternalError,
    ValidationError,
}

impl fmt::Display for AppErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AppErrorCode::InvalidInput => write!(f, "InvalidInput"),
            AppErrorCode::InternalError => write!(f, "InternalError"),
            AppErrorCode::ValidationError => write!(f, "ValidationError"),
        }
    }
}

// The kinds of errors we can hit in our application.
pub enum AppError {
    // Generic server error
    ServerError,
    // The request body contained invalid JSON for the given request
    JsonRejection(JsonRejection),
    // Undefined database error
    DatabaseError(Error),
    // Data sent by user has mistakes
    ValidationError(Vec<FailedValidation>),
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        Self::JsonRejection(rejection)
    }
}

impl From<Error> for AppError {
    fn from(error: Error) -> Self {
        Self::DatabaseError(error)
    }
}

impl From<Vec<FailedValidation>> for AppError {
    fn from(failures: Vec<FailedValidation>) -> Self {
        Self::ValidationError(failures)
    }
}

#[derive(Serialize)]
pub enum ValidationIssue {
    Invalid,
    Required,
    TooWeak,
}

#[derive(Serialize)]
pub struct FailedValidation {
    pub field: String,
    pub issue: ValidationIssue,
}

// Errors can contain extra data that can be used by clients
#[derive(Serialize)]
enum ErrorData {
    #[serde(rename = "")]
    Empty,
    ValidationInfo(Vec<FailedValidation>),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: String,
    message: String,
    data: ErrorData,
}

// Tell axum how `AppError` should be converted into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (response_status, error_code, error_message, error_data) = match self {
            // Generic backend error
            AppError::ServerError => (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppErrorCode::InternalError,
                "We made a mistake. Sorry".to_string(),
                ErrorData::Empty,
            ),
            // This error is caused by bad user input
            AppError::JsonRejection(rejection) => (
                rejection.status(),
                AppErrorCode::InvalidInput,
                "Invalid JSON for this endpoint".to_string(),
                ErrorData::Empty,
            ),
            AppError::DatabaseError(_e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                AppErrorCode::InternalError,
                "We made a mistake. Sorry".to_string(),
                ErrorData::Empty,
            ),
            AppError::ValidationError(data) => (
                StatusCode::BAD_REQUEST,
                AppErrorCode::ValidationError,
                "Request data is invalid".to_string(),
                ErrorData::ValidationInfo(data),
            ),
        };

        (
            response_status,
            Extractor(ErrorResponse {
                code: error_code.to_string(),
                message: error_message,
                data: error_data,
            }),
        )
            .into_response()
    }
}

pub fn internal_error() -> AppError {
    return AppError::ServerError;
}
