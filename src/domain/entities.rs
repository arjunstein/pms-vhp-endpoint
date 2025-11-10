use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Booking {
    pub room_number: String,
    pub password: String,
    pub name: Option<String>,
    pub folio_number: Option<String>,
    pub checkin_date: NaiveDate,
    pub checkout_date: NaiveDate,
    pub gtype: Option<String>,
}
