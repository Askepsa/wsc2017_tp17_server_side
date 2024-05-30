pub use login::*;
pub use logout::*;

pub mod login;
pub mod logout;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct SessionToken {
    token: String,
}
