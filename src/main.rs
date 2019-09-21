use actix_web::{self, web, App, HttpRequest, HttpServer, Responder};
use log;
use simple_logger;

mod ensure;
mod utils;

fn health(_req: HttpRequest) -> impl Responder {
    format!("All ok")
}

fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    HttpServer::new(|| {
        App::new()
            .route("/ensure", web::post().to_async(ensure::execute))
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run()
    .unwrap();
}
