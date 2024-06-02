use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

pub mod search;

#[derive(Debug, Deserialize, Serialize)]
pub struct Schedule {
    id: i32,
    line: i32,
    departure_time: Option<NaiveTime>,
    arrival_time: Option<NaiveTime>,
    from_place_id: i32,
    to_place_id: i32,
}
