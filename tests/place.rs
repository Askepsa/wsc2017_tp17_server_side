mod auth;

use actix_web::test;
use auth::get_session_token;
use wsc2017_tp17::{config::ServerConfig, routes::place::Place};

#[actix_web::test]
async fn getting_all_places_returns_valid_response_for_valid_request() {
    let server_config = ServerConfig::new().await;
    let db_pool = server_config.db_pool.pool.clone();
    let app =
        test::init_service(actix_web::App::new().configure(move |cfg| server_config.config(cfg)))
            .await;

    let token = get_session_token(db_pool.clone()).await;
    let search_params = format!("token={}", token);
    let req = test::TestRequest::get()
        .uri(&format!("/v1/place?{}", search_params))
        .to_request();
    let res: Vec<Place> = test::call_and_read_body_json(&app, req).await;
    let places = sqlx::query_as!(
        Place,
        r#"SELECT id, name, latitude, longitude, x, y, image_path FROM places"#
    )
    .fetch_all(&db_pool)
    .await
    .expect("failed fetching data from places");
    assert!(
        res.iter().all(|data| places.contains(data)),
        "places api does not return every places out there"
    );
}

#[actix_web::test]
async fn getting_all_places_returns_400_for_invalid_request_token() {
    let server_config = ServerConfig::new().await;
    let app =
        test::init_service(actix_web::App::new().configure(move |cfg| server_config.config(cfg)))
            .await;

    let test_cases = vec![
        ("?invalid=forbidden", "invalid search params"),
        ("", "empty search params"),
        ("?token=yep", "invalid token"),
        ("?token=", "empty token"),
    ];

    for (search_params, msg) in test_cases {
        let req = test::TestRequest::get()
            .uri(&format!("/v1/place?{}", search_params))
            .to_request();

        let res = test::call_service(&app, req).await;
        assert!(
            res.status().is_client_error(),
            "{}",
            format!(
                "expecting client error from {}, got {} instead",
                msg,
                res.status()
            )
        );
    }
}
