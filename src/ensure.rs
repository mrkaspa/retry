use crate::utils::GeneralError;
use actix_web::{self, web, HttpRequest};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Payload {
    reference: String,
    retries: u32,
    method: String,
    request_url: String,
    payload: String,
}

pub struct Response {}

pub fn execute(
    (req, json): (HttpRequest, web::Json<Payload>),
) -> actix_web::Result<String, GeneralError> {
    let payload = json.into_inner();
    ensure(payload);
    Err(GeneralError {
        status: 402,
        message: String::from("missing name param"),
    })
}

fn ensure(payload: Payload) {}
