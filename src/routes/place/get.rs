use crate::routes::auth::validate_session_token;
use crate::routes::place::{Place, Places};
use crate::{
    routes::auth::SessionToken,
    routes::{DatabasePool, Res},
};
use actix_web::{web, HttpResponse, Responder};
use sqlx::{Pool, Postgres};

pub async fn get_places(
    req: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let token = req.token.to_owned();

    match validate_session_token(&token, &db_pool.pool).await {
        Err(sqlx::Error::RowNotFound) => HttpResponse::Unauthorized().json(Res {
            msg: "unauthorized".to_owned(),
        }),
        _ => HttpResponse::InternalServerError().json(Res {
            msg: "server err".to_owned(),
        }),
    };

    match query_places(db_pool.pool.clone()).await {
        Ok(places) => HttpResponse::Ok().json(places),
        Err(_) => HttpResponse::InternalServerError().json(Res {
            msg: "server err".to_owned(),
        }),
    }
}

async fn query_places(db_pool: Pool<Postgres>) -> Result<Places, sqlx::Error> {
    let query = sqlx::query_as!(
        Place,
        "SELECT id, name, latitude, longitude, x, y, image_path, description FROM places"
    )
    .fetch_all(&db_pool)
    .await;

    match query {
        Ok(vec) => Ok(Places(vec)),
        Err(err) => Err(err),
    }
}
