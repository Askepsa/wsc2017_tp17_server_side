mod auth;

use actix_web::{http::header::ContentType, test};
use auth::get_session_token;
use sqlx::{Pool, Postgres};
use wsc2017_tp17::{
    config::ServerConfig,
    routes::{auth::SessionToken, place::Place},
};

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
        r#"SELECT id, name, latitude, longitude, x, y, image_path, description FROM places"#
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

#[actix_web::test]
async fn finding_place_returns_200_for_valid_request() {
    let server_config = ServerConfig::new().await;
    let db_pool = server_config.db_pool.pool.clone();
    let app =
        test::init_service(actix_web::App::new().configure(move |cfg| server_config.config(cfg)))
            .await;

    let payload = format!(
        r#"{{"username": "{}", "password": "{}"}}"#,
        "admin", "adminpass"
    );
    let req = test::TestRequest::post()
        .uri("/v1/auth/login")
        .insert_header(ContentType::json())
        .set_payload(payload)
        .to_request();
    let res: SessionToken = test::call_and_read_body_json(&app, req).await;
    let token = res.token;
    let places_id = get_places_id(db_pool.clone()).await;

    for place in places_id {
        let payload = format!("/v1/place/{}?token={}", place.id, token);
        let req = test::TestRequest::get().uri(&payload).to_request();
        let res = test::call_service(&app, req).await;
        assert!(
            res.status().is_success(),
            "{}",
            format!("status: {}\npayload: {}", res.status(), payload)
        );
    }
}

#[actix_web::test]
async fn finding_place_returns_400_for_invalid_request() {
    let server_config = ServerConfig::new().await;
    let db_pool = server_config.db_pool.pool.clone();
    let app =
        test::init_service(actix_web::App::new().configure(move |cfg| server_config.config(cfg)))
            .await;

    let token = get_session_token(db_pool.clone()).await;
    let test_case = vec![
        ("token=yep".to_string(), "invalid token".to_string()),
        ("token=".to_string(), "empty token".to_string()),
        ("".to_string(), "empty search params".to_string()),
        (
            "invalidkey=forbidden".to_string(),
            "invalid search params".to_string(),
        ),
        (
            format!("invalidid?token={}", token),
            "valid token but invalid id".to_string(),
        ),
    ];
    let places_id = get_places_id(db_pool).await;

    // wth am I doing
    for place_id in places_id {
        for (case, msg) in test_case.iter() {
            let payload = format!("{}?{}", place_id.id, case);
            let req = test::TestRequest::get()
                .uri(&format!("/v1/place/{}", payload))
                .to_request();
            let res = test::call_service(&app, req).await;
            assert!(
                res.status().is_client_error(),
                "{}",
                format!(
                    "the server was supposed to return 400 but responded with {} from {} request",
                    res.status(),
                    msg
                )
            );
        }
    }
}

// TODO:
// CREATE Test
// UPDATE Test
// DELETE Test

#[derive(Debug, serde::Deserialize)]
struct PlaceIdSearchParams {
    pub id: i32,
}

async fn get_places_id(db_pool: Pool<Postgres>) -> Vec<PlaceIdSearchParams> {
    sqlx::query_as!(PlaceIdSearchParams, "select id from places;")
        .fetch_all(&db_pool)
        .await
        .expect("unable to query places.id from db")
}
