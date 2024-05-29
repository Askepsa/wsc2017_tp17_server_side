use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;

#[derive(sqlx::Type, Debug, Clone, Copy, Deserialize, Serialize)]
#[sqlx(type_name = "role")]
pub enum Role {
    USER,
    ADMIN,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Res {
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