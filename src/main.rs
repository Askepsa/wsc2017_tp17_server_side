use actix_web::{App, HttpServer};
use anyhow::Result;
use wsc2017_tp17::config::ServerConfig;

#[actix_web::main]
async fn main() -> Result<()> {
    let server_config = ServerConfig::new().await;
    HttpServer::new(move || {
        let server_config = server_config.clone();
        App::new().configure(move |cfg| server_config.config(cfg))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
