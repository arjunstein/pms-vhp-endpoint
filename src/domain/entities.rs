use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Booking {
    pub room_number: String,
    pub password: String,
    pub name: Option<String>,
    pub folio_number: Option<String>,
    pub checkin_date: NaiveDateTime,
    pub checkout_date: NaiveDateTime,
    pub gtype: Option<String>,
}
