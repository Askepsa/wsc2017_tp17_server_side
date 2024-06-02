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
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use std::str::FromStr;

pub async fn shortest_paths(
    slug: web::Path<Slug>,
    query: web::Query<SessionToken>,
    db_pool: web::Data<DatabasePool>,
) -> impl Responder {
    if let Err(err) = validate_session_token(&query.token, &db_pool.pool).await {
        match err {
            sqlx::Error::RowNotFound => {
                return HttpResponse::Unauthorized().json(RouteRes {
                    msg: "unauthorized user".to_string(),
                })
            }
            _ => {
                return HttpResponse::InternalServerError().json(RouteRes {
                    msg: "server err".to_string(),
                })
            }
        }
    }

    let (from_place_id, to_place_id, departure_time) = (
        slug.from_place_id,
        slug.to_place_id,
        slug.departure_time.to_string(),
    );

    let from_place = get_place_by_id(from_place_id, &db_pool.pool).await;
    let to_place = get_place_by_id(to_place_id, &db_pool.pool).await;
    let (from_place, to_place) = match (from_place, to_place) {
        (Ok(f_place), Ok(t_place)) => (f_place, t_place),
        _ => {
            return HttpResponse::InternalServerError().json(RouteRes {
                msg: "server err".to_string(),
            })
        }
    };

    let mock_data = ResponseSchedule {
        id: 1,
        line: 1,
        to_place,
        from_place,
        travel_time: chrono::NaiveTime::from_str("05:00:00")
            .expect("something went wrong converting str to naivetime"),
        arrival_time: chrono::NaiveTime::from_str("05:00:00")
            .expect("something went wrong converting str to naivetime"),
        departure_time: chrono::NaiveTime::from_str("05:00:00")
            .expect("something went wrong converting str to naivetime"),
        from_place_id: 2,
        to_place_id: 3,
    };

    let schedules: Vec<ResponseSchedule> = vec![mock_data];

    HttpResponse::Ok().json(Res { schedules })
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

async fn query_schedule_by_id(
    schedule_id: i32,
    db_pool: &Pool<Postgres>,
) -> Result<Schedule, sqlx::Error> {
    sqlx::query_as!(
        Schedule,
        "SELECT 
            id, 
            line, 
            departure_time, 
            arrival_time, 
            from_place_id, 
            to_place_id 
        FROM schedules WHERE id = $1",
        schedule_id
    )
    .fetch_one(db_pool)
    .await
}

pub async fn magic_algorithm() {
    todo!()
}
