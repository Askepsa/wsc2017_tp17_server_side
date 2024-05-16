use actix_web::{web, App, HttpResponse, HttpServer, Responder};

async fn root() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/", web::get().to(root)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
