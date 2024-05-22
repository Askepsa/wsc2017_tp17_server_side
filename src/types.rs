use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(sqlx::Type, Debug, Clone, Copy, Deserialize, Serialize)]
#[sqlx(type_name = "role")]
pub enum Role {
    USER,
    ADMIN,
}

#[derive(Serialize)]
pub struct OkRes {
    pub token: String,
    pub role: Role,
}

#[derive(Serialize)]
pub struct ErrRes {
    pub msg: String,
}

pub struct DatabasePool {
    pub pool: PgPool,
}

