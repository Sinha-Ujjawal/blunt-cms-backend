use actix_web::{
    error::ResponseError,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Display, Error)]
pub enum MyError {
    DieselError(diesel::result::Error),

    #[display(fmt = "User Already Exists!")]
    UserAlreadyExists,
}

#[derive(Debug, Serialize, Deserialize)]
struct ErrorResponse {
    error: String,
}

impl ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(
                serde_json::json!(ErrorResponse {
                    error: self.to_string(),
                })
                .to_string(),
            )
    }

    fn status_code(&self) -> StatusCode {
        use MyError::*;
        match *self {
            DieselError(_) => StatusCode::BAD_REQUEST,
            UserAlreadyExists => StatusCode::BAD_REQUEST,
        }
    }
}
