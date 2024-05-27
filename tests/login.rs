use actix_web::{http::header::ContentType, test, App};
use serde::{Deserialize, Serialize};
use wsc2017_tp17::config;

#[derive(Serialize, Deserialize, Debug)]
struct Res {
    token: String,
    role: String,
}

#[actix_web::test]
async fn test_login() {
    let server_config = config::ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let payload = r#"{"username": "admin", "password": "adminpass"}"#;

    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();

    let _: Res = test::call_and_read_body_json(&app, req).await;
}
