pub use login::*;
pub use logout::*;

pub mod login;
pub mod logout;

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[derive(Deserialize)]
pub struct SessionToken {
    pub token: String,
}

#[derive(Serialize)]
pub struct Session {
    pub token: String,
    pub username: String,
}

pub async fn validate_session_token(
    session_token: &str,
    db_pool: &Pool<Postgres>,
) -> Result<(), sqlx::Error> {
    let query = sqlx::query!("SELECT token FROM sessions WHERE token = $1", session_token)
        .fetch_one(db_pool)
        .await;

    match query {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    }
}
