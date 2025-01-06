use axum::{
    extract::FromRequest,
    response::{IntoResponse, Response},
};

use crate::error::app_error::AppError;

// Our own JSON extractor that wraps `axum::Json`. This way we can override
// json rejections and provide an error in whichever format we want
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct Extractor<T>(pub T);

impl<T> IntoResponse for Extractor<T>
where
    axum::Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        axum::Json(self.0).into_response()
    }
}
