use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct UserCreds {
    username: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
pub struct ErrMsg {
    message: String,
}

pub async fn login(request: Option<web::Json<UserCreds>>) -> impl Responder {
    if let Some(req) = request {
        return HttpResponse::Ok().json(req);
    }

    HttpResponse::BadRequest().json(ErrMsg {
        message: String::from("invalid login"),
    })
}
