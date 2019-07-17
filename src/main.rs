use actix_web::{
    self,  web, App, HttpRequest, HttpServer, Responder,
};

mod ensure;
mod utils;

fn health(_req: HttpRequest) -> impl Responder {
    format!("All ok")
}

fn main() {
    HttpServer::new(|| {
        App::new()
            .route("/ensure", web::get().to(ensure::execute))
            .route("/health", web::get().to(health))
    })
    .bind("127.0.0.1:8000")
    .expect("Can not bind to port 8000")
    .run()
    .unwrap();
}
