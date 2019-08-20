use actix_web::{self, error, http::StatusCode, HttpResponse};
use failure::Fail;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Debug, Serialize, Fail)]
#[fail(display = "my error")]
pub struct GeneralError {
    pub status: u16,
    pub message: String,
}

impl error::ResponseError for GeneralError {
    fn render_response(&self) -> HttpResponse {
        HttpResponse::build(StatusCode::from_u16(self.status).unwrap()).json2(self)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RetryError {
    pub retry_no: u32,
}

impl fmt::Display for RetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RetryError is here!")
    }
}

impl Error for RetryError {}
