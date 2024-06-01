use crate::routes::{
    auth::{validate_session_token, SessionToken},
    DatabasePool,
};
use actix_web::{web, HttpResponse, Responder};

use super::Slug;

pub async fn delete_place(
    slug: web::Path<Slug>,
    search_param: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    if let Err(err) = validate_session_token(&search_param.token, &db_pool.pool).await {
        match err {
            sqlx::Error::RowNotFound => return HttpResponse::Unauthorized(),
            _ => return HttpResponse::InternalServerError(),
        }
    }

    let query = sqlx::query!("DELETE FROM places WHERE id = $1", slug.id)
        .execute(&db_pool.pool)
        .await;

    match query {
        Ok(_) => HttpResponse::Ok(),
        _ => HttpResponse::InternalServerError(),
    }
}
