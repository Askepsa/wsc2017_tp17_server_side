use actix_web::{http::header::ContentType, test, App};
use wsc2017_tp17::config::ServerConfig;
use wsc2017_tp17::routes::auth::login;

#[actix_web::test]
async fn login_returns_200_for_valid_request() {
    let server_config = ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let payload = r#"{"username": "admin", "password": "adminpass"}"#;
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "{}",
        format!("expecting success response but got {}", res.status())
    );
}

#[actix_web::test]
async fn login_returns_400_for_invalid_request() {
    let server_config = ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let payload = r#"{"username": "admin", "password": "adminpassz"}"#;
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_client_error(),
        "{}",
        format!(
            "expecting client error response but got {} from {} request",
            res.status(),
            payload
        )
    );
}

#[actix_web::test]
async fn login_returns_400_for_missing_request() {
    let server_config = ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let payload = r#""#;
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_client_error(),
        "{}",
        format!(
            "expecting client error response but got {} from {} request",
            res.status(),
            payload
        )
    );
}

#[actix_web::test]
async fn logout_returns_200_for_valid_request() {
    let server_config = ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    // get token
    let payload = r#"{"username": "admin", "password": "adminpass"}"#;
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();

    let login_res: login::OkRes = test::call_and_read_body_json(&app, req).await;
    let token = login_res.token;

    // test logout api
    let url_params = format!(r#"token={}"#, token);
    let req = test::TestRequest::get()
        .uri(&format!("/v1/auth/logout?{}", url_params))
        .to_request();

    let res = test::call_service(&app, req).await;

    assert!(
        res.status().is_success(),
        "{}",
        format!(
            "expecting client error response but got {} from {} request",
            res.status(),
            payload
        )
    );
}

#[actix_web::test]
async fn logout_returns_400_for_valid_request() {
    let server_config = ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    // get token
    let payload = r#"{"username": "admin", "password": "adminpass"}"#;
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();

    let login_res: login::OkRes = test::call_and_read_body_json(&app, req).await;
    let token = login_res.token;

    // test logout api
    let payload = format!(r#""token": "{}""#, token);
    let req = test::TestRequest::post()
        .uri("/v1/auth/logout")
        .insert_header(ContentType::json())
        .set_payload(payload.clone())
        .to_request();

    let res = test::call_service(&app, req).await;

    assert!(
        res.status().is_client_error(),
        "{}",
        format!(
            "expecting client error response but got {} from {} request",
            res.status(),
            payload
        )
    );
}
