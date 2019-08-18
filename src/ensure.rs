use crate::utils::{GeneralError, RetryError};
use actix_http::http::{header, Method};
use actix_http::RequestHead;
use actix_web::client::Client;
use actix_web::{self, web, HttpRequest, HttpResponse};
use futures::future::{lazy, Future};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct RetryPayload {
    reference: String,
    retries: u32,
    method: String,
    headers: Vec<(String, String)>,
    request_url: String,
    payload: String,
}

#[derive(Serialize)]
pub struct RetryResponse {
    message: String,
}

pub fn execute(json: web::Json<RetryPayload>) -> actix_web::Result<HttpResponse, GeneralError> {
    let payload = json.into_inner();
    match ensure(payload) {
        Ok(()) => Ok(HttpResponse::Ok().json(RetryResponse {
            message: String::from("all ok"),
        })),
        Err(err) => Err(GeneralError {
            status: 402,
            message: format!("Failed will retry {} times", err.retry_no),
        }),
    }
}

fn ensure(payload: RetryPayload) -> Result<(), RetryError> {
    let mut sys = actix::System::new("request");
    let retries = payload.retries;
    sys.block_on(lazy(|| {
        let client = Client::default();
        let mut req_head = RequestHead::default();
        for tuple in payload.headers.iter() {
            let (header_, value) = tuple.clone();
            req_head.headers.append(
                header::HeaderName::from_bytes(&header_.into_bytes()).unwrap(),
                header::HeaderValue::from_bytes(&value.into_bytes()).unwrap(),
            );
        }
        req_head.method = Method::from_bytes(&payload.method.into_bytes()).unwrap();
        client
            .request_from(payload.request_url, &req_head)
            .send_body(payload.payload)
            .map_err(|_| RetryError { retry_no: retries })
            .and_then(|response| {
                // <- server http response
                println!("Response: {:?}", response);
                Ok(())
            })
    }))
}
