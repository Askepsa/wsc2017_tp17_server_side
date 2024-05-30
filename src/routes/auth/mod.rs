pub use login::*;
pub use logout::*;

pub mod login;
pub mod logout;

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct SessionToken {
    pub token: String,
}

#[derive(Serialize)]
pub struct Session {
    pub token: String,
    pub username: String,
}
