use crate::database::DatabasePool;
use actix_web::{web, HttpResponse, Responder};
use jsonwebtoken::{self, encode, EncodingKey, Header};
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

    let (username, role) = match &query_res {
        Ok(res) => (res.username.clone(), res.role.clone()),
        _ => {
            return HttpResponse::InternalServerError().json(ErrRes {
                msg: "database boom".into(),
            })
        }
    };

    let claim = Claim {
        username,
        role,
        exp: 10000000000, // di ko alam kung relative ba to o absolute
    };

    let token = match encode(
        &Header::default(),
        &claim,
        &EncodingKey::from_secret(JW_TOKEN_KEY.as_ref()),
    ) {
        Ok(token) => token,
        _ => {
            return HttpResponse::InternalServerError().json(ErrRes {
                msg: "jwt boom".into(),
            })
        }
    };

    return HttpResponse::Ok().json(OkRes { token, role });
}

const JW_TOKEN_KEY: &'static str = "averysafekey";

#[derive(Deserialize, Serialize)]
struct Claim {
    username: String,
    role: Role,
    exp: usize,
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
