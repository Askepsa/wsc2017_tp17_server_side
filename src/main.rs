use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use wsc2017_tp17::routes::auth::login;

async fn root() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().route("/", web::get().to(root)).service(
            web::scope("/v1").service(
                web::scope("/auth")
                    .route("/login", web::post().to(login))
            ),
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
