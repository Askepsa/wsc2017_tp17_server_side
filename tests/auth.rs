use actix_web::{http::header::ContentType, test, App};
use sqlx::{Pool, Postgres};
use wsc2017_tp17::config::ServerConfig;

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
async fn login_returns_400_for_invalid_requests() {
    let server_config = ServerConfig::new().await;
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let payloads: Vec<(String, &str)> = vec![
        (
            r#"{"username": "adminz", "password": "adminpass"}"#.to_string(),
            "invalid username",
        ),
        (
            r#"{"username": "admin", "password": "adminpassz"}"#.to_string(),
            "invalid password",
        ),
        (
            r#"{"username": "", "password": "adminpass"}"#.to_string(),
            "empty username",
        ),
        (
            r#"{"username": "admin", "password": ""}"#.to_string(),
            "empty password",
        ),
        (
            r#"{"username": "admin", "": ""}"#.to_string(),
            "empty username and password",
        ),
    ];

    for (payload, err_msg) in payloads {
        let req = test::TestRequest::post()
            .uri("/v1/auth/login")
            .insert_header(ContentType::json())
            .set_payload(payload.clone())
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(
            res.status().is_client_error(),
            "{}",
            format!(
                "expecting client error response when the request has an {}, but instead it responded with {}",
                err_msg,
                res.status()
            )
        );
    }
}

#[actix_web::test]
async fn logout_returns_200_for_valid_request() {
    let server_config = ServerConfig::new().await;
    let db_pool = server_config.db_pool.pool.clone();
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let token = get_token(db_pool).await;
    let url_params = format!(r#"token={}"#, token);
    let req = test::TestRequest::get()
        .uri(&format!("/v1/auth/logout?{}", url_params))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "{}",
        format!("expecting client error response but got {}", res.status())
    );
}

#[actix_web::test]
async fn logout_returns_400_for_invalid_request() {
    let server_config = ServerConfig::new().await;
    let db_pool = server_config.db_pool.pool.clone();
    let app = test::init_service(App::new().configure(move |cfg| server_config.config(cfg))).await;

    let token = get_token(db_pool).await;
    let url_params = format!(r#""token": "{}""#, token);
    let req = test::TestRequest::post()
        .uri("/v1/auth/logout")
        .insert_header(ContentType::json())
        .set_payload(url_params.clone())
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_client_error(),
        "{}",
        format!(
            "expecting client error response but got {} from {} request",
            res.status(),
            url_params
        )
    );
}

async fn get_token(db_pool: Pool<Postgres>) -> String {
    sqlx::query!("SELECT TOKEN FROM sessions LIMIT 1;")
        .fetch_one(&db_pool)
        .await
        .expect("unable to fetch token from database")
        .token
        .expect("failed parsing record to string")
}
