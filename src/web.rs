use crate::ensure::ensure;
use crate::structs::RetryPayload;
use crate::utils::GeneralError;
use actix_web::{self, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use futures::future::Future;
use log::{self, info};
use serde::Serialize;
use simple_logger;
mod ensure;
mod rabbit;
mod structs;
mod utils;

fn health(_req: HttpRequest) -> impl Responder {
    format!("All ok")
}

#[derive(Serialize)]
struct RetryResponse {
    message: String,
}

fn execute(
    json: web::Json<RetryPayload>,
) -> impl Future<Item = HttpResponse, Error = GeneralError> {
    info!("Request arrived");
    let payload = json.into_inner();
    ensure(payload)
        .and_then(|_| {
            Ok(HttpResponse::Ok().json(RetryResponse {
                message: String::from("all ok"),
            }))
        })
        .map_err(|err| GeneralError {
            status: 402,
            message: format!("Failed will retry {} times", err.retry_no),
        })
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    HttpServer::new(|| {
        App::new()
            .route("/ensure", web::post().to_async(execute))
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run()
    .unwrap();
}
