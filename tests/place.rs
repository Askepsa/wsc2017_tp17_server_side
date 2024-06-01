mod auth;

use actix_web::test;
use auth::get_session_token;
use sqlx::{Pool, Postgres};
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

// #[actix_web::test]
// async fn finding_place_returns_valid_place_for_valid_request() {
//     let server_config = ServerConfig::new().await;
//     let db_pool = server_config.db_pool.pool.clone();
//     let app =
//         test::init_service(actix_web::App::new().configure(move |cfg| server_config.config(cfg)))
//             .await;
//
//     let token = get_session_token(db_pool.clone()).await;
//     let queries = get_places_id(db_pool.clone()).await;
//
//     for query in queries {
//         let req = test::TestRequest::get()
//             .uri(&format!("/v1/place/{}?token={}", token, query.id))
//             .to_request();
//         let res: Place = test::call_and_read_body_json(&app, req).await;
//         let place = sqlx::query_as!(
//             Place,
//             r#"SELECT id, name, latitude, longitude, x, y, image_path FROM places WHERE id = $1"#,
//             query.id
//         )
//         .fetch_one(&db_pool)
//         .await
//         .expect("failed fetching place from db");
//
//         assert_eq!(res, place);
//     }
// }

#[actix_web::test]
async fn finding_place_returns_400_for_invalid_request() {
    let server_config = ServerConfig::new().await;
    let db_pool = server_config.db_pool.pool.clone();
    let app =
        test::init_service(actix_web::App::new().configure(move |cfg| server_config.config(cfg)))
            .await;

    let invalid_token_payload_test_cases = vec![
        ("invalidkey=forbidden", "invalid search params"),
        ("token=yep", "invalid token"),
        ("token=", "empty token"),
        ("", "empty search params"),
    ];
    let queries = sqlx::query_as!(PlaceIdSearchParams, "SELECT id from places;")
        .fetch_all(&db_pool.clone())
        .await
        .expect("unable to query id from places db");
    println!("{:?}", queries);

    for query in queries {
        for (invalid_token_payload, msg) in invalid_token_payload_test_cases.iter() {
            let payload = format!("{}?{}", query.id, invalid_token_payload);
            println!("{}", payload);

            let req = test::TestRequest::get()
                .uri(&format!("v1/place/{}", payload))
                .to_request();
            let res = test::call_service(&app, req).await;

            // assert if the server is returning 400
            // assert!(
            //     res.status().is_client_error(),
            //     "{}",
            //     format!(
            //         "the server was supposed to return 400 but responded with {} from {} request",
            //         res.status(),
            //         msg
            //     )
            // );
        }
    }

    // let token = get_session_token(db_pool.clone()).await;
    // let (invalid_id_payload, msg) = (
    //     format!("69420?token={}", token),
    //     "invalid k, v search parameter",
    // );

    // let req = test::TestRequest::get()
    //     .uri(&format!("v1/place/{}", invalid_id_payload))
    //     .to_request();
    // let res = test::call_service(&app, req).await;
    //
    // assert!(
    //     res.status().is_client_error(),
    //     "{}",
    //     format!(
    //         "the server was supposed to return 400 but responded with {} from {} request",
    //         res.status(),
    //         msg
    //     ),
    // )
}

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
