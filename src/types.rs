use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(sqlx::Type, Debug, Clone, Copy, Deserialize, Serialize)]
#[sqlx(type_name = "role")]
pub enum Role {
    USER,
    ADMIN,
}

#[derive(Deserialize, Serialize)]
pub struct ErrMsg {
    pub msg: String,
}

#[derive(Deserialize, Serialize)]
pub struct OkMsg {
    pub msg: String,
}

pub struct DatabasePool {
    pub pool: PgPool,
}

#[derive(Serialize)]
pub struct Session {
    pub token: String,
    pub username: String,
}
