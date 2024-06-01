use crate::routes::auth::SessionToken;
use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;

// Route Search (v1/route/search/{FROM_PLACE_ID}/{TO_PLACE_ID}/[DEPARTURE_TIME]?token={AUTHORIZATION_TOKEN})
// URL: http://<server>/<XX>_Server_A/api/v1/route/search/5/1/13:00:00?token={authorization_token}

#[derive(Deserialize)]
pub struct Slug {
    from_place_id: String,
    to_place_id: String,
    departure_time: String,
}

pub async fn get_shortest_paths(
    slug: web::Path<Slug>,
    query: web::Query<SessionToken>,
) -> impl Responder {
    let (from_place_id, to_place_id, departure_time) = (
        slug.from_place_id.to_owned(),
        slug.to_place_id.to_owned(),
        slug.departure_time.to_owned(),
    );

    println!("{} {} {}", from_place_id, to_place_id, departure_time);
    println!("Token: {}", query.token);

    HttpResponse::Ok()
}

pub async fn magic_algorithm() {
    todo!()
}
