use rocket::serde::json::{json, Value};
use rocket::serde::{Serialize, Deserialize};

// import services module
use crate::services;

// create a struct to hold our Date data
// need serialize/deserialize to convert to/from JSON
#[derive(Debug, Deserialize, Serialize)]
pub struct Date {
    pub day: u32,
    pub month: u32,
    pub year: i32
}

// create get-current-date route under /date and call get_current_date service which will return a Date object
// route returns a Date object converted to JSON
#[get("/date/get-current-date")]
pub fn get_current_date() -> Value {
    json!(services::date::get_current_date())
}