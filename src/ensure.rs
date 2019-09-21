use crate::structs::RetryPayload;
use crate::utils::RetryError;
use actix_http::http::{header, Method};
use actix_http::RequestHead;
use actix_web::client::Client;
use futures::future::Future;
use log::{error, info};

pub fn ensure(payload: RetryPayload) -> impl Future<Item = (), Error = RetryError> {
    send(payload).map_err(|error| {
        if error.retry_no > 0 {
            // TODO send in amqp
            info!("Sending in Rabbit")
        }
        error
    })
}

fn send(payload: RetryPayload) -> impl Future<Item = (), Error = RetryError> {
    let retries = payload.retries;

    let mut req_head = RequestHead::default();
    for tuple in payload.headers.iter() {
        let (header_, value) = tuple.clone();
        req_head.headers.append(
            header::HeaderName::from_bytes(&header_.into_bytes()).unwrap(),
            header::HeaderValue::from_bytes(&value.into_bytes()).unwrap(),
        );
    }
    req_head.method = Method::from_bytes(&payload.method.into_bytes()).unwrap();

    Client::default()
        .request_from(payload.request_url, &req_head)
        .send_body(payload.payload)
        .map_err(move |err| {
            error!("Error doing request {}", err);
            RetryError { retry_no: retries }
        })
        .and_then(|response| {
            // <- server http response
            info!("Response: {:?}", response);
            Ok(())
        })
}
