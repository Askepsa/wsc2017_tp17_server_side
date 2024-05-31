use crate::routes::auth::SessionToken;
use actix_web::{web, HttpResponse, Responder};
use sqlx::{Error, Pool, Postgres};

use crate::routes::{DatabasePool, Res};

pub async fn logout(
    uri_req: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let token = uri_req.token.to_owned();

    if (perform_session_deletion(token, &db_pool.pool).await).is_err() {
        return HttpResponse::InternalServerError().json(Res { msg: "yep".into() });
    }

    HttpResponse::Ok().json(Res {
        msg: "logout success".to_string(),
    })
}

async fn perform_session_deletion(token: String, db_pool: &Pool<Postgres>) -> Result<(), Error> {
    let _ = sqlx::query!("DELETE from sessions where token = $1", token)
        .execute(db_pool)
        .await?;

    Ok(())
}
