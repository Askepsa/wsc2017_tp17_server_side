use actix_web::{web, HttpResponse, Responder};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{Error, Pool, Postgres};

use crate::types::{DatabasePool, Role};

#[derive(Debug, Deserialize, Serialize)]
pub struct OkRes {
    pub token: String,
    pub role: Role,
}

#[derive(Serialize)]
pub struct ErrRes {
    pub msg: String,
}

#[derive(Serialize)]
pub struct UserCreds {
    username: String,
    role: Role,
}

#[derive(Deserialize)]
pub struct UserRequest {
    username: String,
    password: String,
}

pub async fn login(
    request: Option<web::Json<UserRequest>>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let (username, password) = if let Some(val) = &request {
        (val.username.clone(), val.password.clone())
    } else {
        return HttpResponse::BadRequest().json(ErrRes {
            msg: "invalid login".into(),
        });
    };

    let query_res = get_user_creds(&username, &password, &db_pool.pool).await;
    let (username, role) = match &query_res {
        Ok(res) => (res.username.clone(), res.role),
        Err(err) => match *err {
            sqlx::Error::RowNotFound => {
                return HttpResponse::BadRequest().json(ErrRes { msg: "invalid login".into() })
            }
            _ => {
                return HttpResponse::InternalServerError().json(ErrRes {
                    msg: "database boom".into(),
                })
            }
        },
    };

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(100)
        .map(char::from)
        .collect();

    if (insert_into_session(&token, &username, &db_pool.pool).await).is_err() {
        return HttpResponse::InternalServerError().json(ErrRes {
            msg: "database boom".into(),
        });
    }

    HttpResponse::Ok().json(OkRes { token, role })
}

async fn get_user_creds(
    username: &str,
    password: &str,
    db_pool: &Pool<Postgres>,
) -> Result<UserCreds, Error> {
    sqlx::query_as!(
        UserCreds,
        "SELECT username, role as \"role: Role\" FROM users WHERE username = $1 AND password = $2",
        username,
        password
    )
    .fetch_one(db_pool)
    .await
}

async fn insert_into_session(
    token: &str,
    username: &str,
    db_pool: &Pool<Postgres>,
) -> Result<sqlx::postgres::PgQueryResult, Error> {
    sqlx::query_as!(
        Session,
        "INSERT INTO sessions VALUES ($1, $2)",
        token,
        username
    )
    .execute(db_pool)
    .await
}
