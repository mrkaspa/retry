use actix_web::{self, error, http::StatusCode, HttpResponse};
use failure::Fail;
use serde::Serialize;

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
