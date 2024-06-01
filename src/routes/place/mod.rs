pub mod get;
pub mod slug;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
pub struct Place {
    pub id: i32,
    pub name: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub image_path: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Places(pub Vec<Place>);
