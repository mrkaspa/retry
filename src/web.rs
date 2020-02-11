use crate::ensure::ensure;
use crate::structs::RetryPayload;
use crate::utils::GeneralError;
use actix_web::{self, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use log::{self, info};
use serde::Serialize;
use simple_logger;
mod ensure;
mod rabbit;
mod structs;
mod utils;

async fn health(_req: HttpRequest) -> impl Responder {
    format!("All ok")
}

#[derive(Serialize)]
struct RetryResponse {
    message: String,
}

async fn execute(json: web::Json<RetryPayload>) -> Result<HttpResponse, GeneralError> {
    info!("Request arrived");
    let payload = json.into_inner();
    ensure(payload)
        .await
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

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    HttpServer::new(|| {
        App::new()
            .route("/ensure", web::post().to(execute))
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run()
    .await
}
