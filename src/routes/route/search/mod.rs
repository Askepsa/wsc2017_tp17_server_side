#![allow(unused)]
use super::super::Res as RouteRes;
use crate::routes::{
    auth::{validate_session_token, SessionToken},
    place::{slug::get::get_place_by_id, Place},
    route::Schedule,
    DatabasePool,
};
use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveTime;
use graph::Graph;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::str::FromStr;

pub mod graph;

pub async fn shortest_paths(
    slug: web::Path<Slug>,
    // query: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    // if let Err(err) = validate_session_token(&query.token, &db_pool.pool).await {
    //     match err {
    //         sqlx::Error::RowNotFound => {
    //             return HttpResponse::Unauthorized().json(RouteRes {
    //                 msg: "unauthorized user".to_string(),
    //             })
    //         }
    //         _ => {
    //             return HttpResponse::InternalServerError().json(RouteRes {
    //                 msg: "server err".to_string(),
    //             })
    //         }
    //     }
    // }

    unsafe {
        let graph = Graph::new(db_pool.pool.clone(), &slug.departure_time)
            .await
            .expect("Sumabog ang paggawa ng graph");
    }

    HttpResponse::Ok().json("haha")
}

#[derive(Deserialize)]
pub struct Slug {
    from_place_id: i32,
    to_place_id: i32,
    departure_time: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResponseSchedule {
    id: i32,
    line: i32,
    from_place_id: i32,
    to_place_id: i32,
    departure_time: NaiveTime,
    arrival_time: NaiveTime,
    travel_time: NaiveTime,
    from_place: Place,
    to_place: Place,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Res {
    schedules: Vec<ResponseSchedule>,
}

pub async fn magic_algorithm() {
    todo!()
}
