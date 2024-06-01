use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{error, Pool, Postgres};

use crate::routes::{
    auth::{validate_session_token, SessionToken},
    place::Place,
    DatabasePool, Res,
};

#[derive(Deserialize)]
pub struct Slug {
    id: i32,
}

pub async fn find_place(
    slug: web::Path<Slug>,
    search_param: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    // get place
    let place = match get_place_by_id(slug.id, &db_pool.pool).await {
        Ok(place) => place,
        Err(err) => match err {
            error::Error::RowNotFound => {
                return HttpResponse::BadRequest().json(Res {
                    msg: "invalid request".to_string(),
                });
            }
            _ => {
                return HttpResponse::InternalServerError().json(Res {
                    msg: "my fault".to_string(),
                })
            }
        },
    };

    // validate sesion_token
    if let Err(err) = validate_session_token(&search_param.token, &db_pool.pool).await {
        match err {
            error::Error::RowNotFound => {
                return HttpResponse::BadRequest().json(Res {
                    msg: "unauthorized".to_string(),
                })
            }
            _ => {
                return HttpResponse::InternalServerError().json(Res {
                    msg: "my fault".to_string(),
                })
            }
        };
    }

    HttpResponse::Ok().json(place)
}

async fn get_place_by_id(place_id: i32, db_pool: &Pool<Postgres>) -> Result<Place, sqlx::Error> {
    let query = sqlx::query_as!(
        Place,
        "SELECT id, name, latitude, longitude, x, y, image_path FROM places where id = $1",
        place_id
    )
    .fetch_one(db_pool)
    .await;

    match query {
        Ok(query) => Ok(query),
        Err(err) => Err(err),
    }
}
