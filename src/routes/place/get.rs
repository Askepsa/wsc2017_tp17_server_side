use crate::routes::place::{Place, Places};
use crate::{
    routes::auth::SessionToken,
    routes::{DatabasePool, Res},
};
use actix_web::{web, HttpResponse, Responder};
use sqlx::{Pool, Postgres};

pub async fn get_places(
    req: Option<web::Query<SessionToken>>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let token = if let Some(req) = req {
        req.token.clone()
    } else {
        return HttpResponse::BadRequest().json(Res {
            msg: "Unauthorized user".to_owned(),
        });
    };

    if validate_token(&token, db_pool.clone().pool.clone())
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().json(Res {
            msg: "server err".to_owned(),
        });
    }

    match query_places(db_pool.pool.clone()).await {
        Ok(places) => HttpResponse::Ok().json(places),
        Err(_) => HttpResponse::InternalServerError().json(Res {
            msg: "server err".to_owned(),
        }),
    }
}

async fn validate_token(token: &str, db_pool: Pool<Postgres>) -> Result<(), sqlx::Error> {
    let query = sqlx::query!("SELECT token FROM sessions WHERE token = $1", token)
        .fetch_one(&db_pool)
        .await;

    match query {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}

async fn query_places(db_pool: Pool<Postgres>) -> Result<Places, sqlx::Error> {
    let query = sqlx::query_as!(
        Place,
        "SELECT id, name, latitude, longitude, x, y, image_path FROM places"
    )
    .fetch_all(&db_pool)
    .await;

    match query {
        Ok(vec) => Ok(Places(vec)),
        Err(err) => Err(err),
    }
}
