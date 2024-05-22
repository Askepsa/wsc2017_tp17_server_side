use crate::database::DatabasePool;
use actix_web::{web, HttpResponse, Responder};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

pub async fn login(
    request: Option<web::Json<UserRequest>>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    let (username, password) = match &request {
        Some(val) => (val.username.clone(), val.password.clone()),
        _ => {
            return HttpResponse::BadRequest().json(ErrRes {
                msg: "invalid login".into(),
            })
        }
    };

    let query_res = sqlx::query_as!(
        UserCreds,
        "SELECT username, role as \"role: Role\" FROM users WHERE username = $1 AND password = $2",
        username,
        password
    )
    .fetch_one(&db_pool.pool)
    .await;

    let token: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(100)
        .map(char::from)
        .collect();

    let (username, role) = match &query_res {
        Ok(res) => (res.username.clone(), res.role.clone()),
        _ => {
            return HttpResponse::InternalServerError().json(ErrRes {
                msg: "database boom".into(),
            })
        }
    };

    let query = sqlx::query_as!(
        Session,
        "INSERT INTO sessions VALUES ($1, $2)",
        token.clone(),
        username
    )
    .execute(&db_pool.pool)
    .await;

    if let Err(_) = query {
        return HttpResponse::InternalServerError().json(ErrRes {
            msg: "database boom".into(),
        });
    }

    return HttpResponse::Ok().json(OkRes { token, role });
}
#[derive(sqlx::Type, Debug, Clone, Copy, Deserialize, Serialize)]
#[sqlx(type_name = "role")]
enum Role {
    USER,
    ADMIN,
}

#[derive(Deserialize, Serialize)]
pub struct UserCreds {
    username: String,
    role: Role,
}

#[derive(Deserialize, Serialize)]
pub struct Session {
    token: String,
    username: String,
}

#[derive(Deserialize)]
pub struct UserRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct OkRes {
    token: String,
    role: Role,
}

#[derive(Deserialize, Serialize)]
struct ErrRes {
    msg: String,
}
