use actix_web::{App, HttpServer};
use anyhow::Result;
use std::net::SocketAddr;
use wsc2017_tp17::config::ServerConfig;

#[actix_web::main]
async fn main() -> Result<()> {
    let server_config = ServerConfig::new().await;
    let host = server_config
        .env
        .get("HOST")
        .expect("server host from env var not found");
    let port = server_config
        .env
        .get("PORT")
        .expect("server port from env var not found");
    let server_addr = format!("{}:{}", host, port);

    HttpServer::new(move || {
        let server_config = server_config.clone();
        App::new().configure(move |cfg| server_config.config(cfg))
    })
    .bind(server_addr.parse::<SocketAddr>().unwrap())?
    .run()
    .await?;

    Ok(())
}
