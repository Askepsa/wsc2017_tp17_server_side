use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use sqlx::{Error, Pool, Postgres};

use crate::types::{DatabasePool, ErrMsg, OkMsg};

#[derive(Deserialize)]
pub struct URLSearchParams {
    token: String,
}

pub async fn logout(
    uri_req: Option<web::Query<URLSearchParams>>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let token = if let Some(req) = uri_req {
        req.token.clone()
    } else {
        return HttpResponse::BadRequest().json(ErrMsg { msg: "yep".into() });
    };

    if (perform_session_deletion(token, &db_pool.pool).await).is_err() {
        return HttpResponse::BadRequest().json(ErrMsg { msg: "yep".into() });
    }

    HttpResponse::Ok().json(OkMsg {
        msg: "logout success".to_string(),
    })
}

async fn perform_session_deletion(token: String, db_pool: &Pool<Postgres>) -> Result<(), Error> {
    let _ = sqlx::query!("DELETE from sessions where token = $1", token)
        .execute(db_pool)
        .await?;

    Ok(())
}
