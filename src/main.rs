use actix_web::{
    web::{self, Data},
    App, HttpResponse, HttpServer, Responder,
};
use sqlx::postgres::PgPoolOptions;
use wsc2017_tp17::{
    routes::auth::{login, logout},
    types::DatabasePool,
};

async fn root() -> impl Responder {
    HttpResponse::Ok()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_pool: Data<DatabasePool> = web::Data::new(DatabasePool {
        pool: PgPoolOptions::new()
            .connect("postgresql://postgres:@localhost:5432/wsc")
            .await
            .expect("Connecting to database failed"),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(db_pool.clone())
            .route("/yep", web::post().to(root))
            .service(
                web::scope("/v1").service(
                    web::scope("/auth")
                        .route("/login", web::post().to(login))
                        .route("/logout", web::get().to(logout)),
                ),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
