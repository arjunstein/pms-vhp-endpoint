use crate::application::dtos::{PmsQuery, PmsResponse};
use crate::domain::{entities::Booking, repositories::BookingRepository};
use anyhow::{Result, anyhow};
use chrono::{NaiveDate, NaiveTime};
use std::sync::Arc;

pub struct BookingService<R: BookingRepository> {
    repo: Arc<R>,
}

impl<R: BookingRepository> BookingService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn process(&self, query: PmsQuery) -> Result<PmsResponse> {
        match query.mode.as_str() {
            "checkin" => self.handle_checkin(query).await,
            _ => Err(anyhow!("invalid mode")),
        }
    }

    async fn handle_checkin(&self, query: PmsQuery) -> Result<PmsResponse> {
        // --- 1️⃣ Input validation ---
        let room = query.room.clone().ok_or_else(|| anyhow!("missing room"))?;
        let pass = query.pass.clone().ok_or_else(|| anyhow!("missing pass"))?;
        let cidate = query
            .cidate
            .clone()
            .ok_or_else(|| anyhow!("missing checkin date"))?;
        let codate = query
            .codate
            .clone()
            .ok_or_else(|| anyhow!("missing checkout date"))?;

        // --- 2️⃣ Parse date/time ---
        let check_in_date = NaiveDate::parse_from_str(&cidate, "%d/%m/%Y")
            .map_err(|_| anyhow!("invalid checkin date format"))?;
        let check_out_date = NaiveDate::parse_from_str(&codate, "%d/%m/%Y")
            .map_err(|_| anyhow!("invalid checkout date format"))?;

        let _check_out_time = if let Some(time_str) = &query.cotime {
            Some(
                NaiveTime::parse_from_str(time_str, "%H:%M:%S")
                    .map_err(|_| anyhow!("invalid checkout time format"))?,
            )
        } else {
            None
        };

        // --- 3️⃣ Create domain entity Booking ---
        let booking = Booking {
            room_number: room,
            password: pass,
            name: query.name.clone(),
            checkin_date: check_in_date,
            checkout_date: check_out_date,
            folio_number: query.rsvno.clone(),
            gtype: query.gtype,
        };

        // --- 4️⃣ Save to DB via repository ---
        self.repo.checkin_repo(&booking).await?;

        // --- 5️⃣ Response ---
        Ok(PmsResponse {
            status: "success".into(),
            message: format!("Room {} successfully checked in", booking.room_number),
        })
    }
}
