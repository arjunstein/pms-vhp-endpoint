use crate::application::dtos::{PmsQueryParams, PmsResponse};
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

    pub async fn process(&self, query: PmsQueryParams) -> Result<PmsResponse> {
        match query.mode.as_str() {
            "checkin" => self.handle_checkin(query).await,
            "checkout" => self.handle_checkout(query).await,
            mode => {
                tracing::warn!("Invalid mode provided: {}", mode);
                Ok(PmsResponse {
                    status: "error".into(),
                    message: "invalid mode".into(),
                })
            }
        }
    }

    async fn handle_checkin(&self, query: PmsQueryParams) -> Result<PmsResponse> {
        // --- 1️⃣ Input validation ---
        let room = match query.room.clone() {
            Some(r) if !r.is_empty() => r,
            _ => {
                return Ok(PmsResponse {
                    status: "error".into(),
                    message: "room is required".into(),
                });
            }
        };
        let pass = match query.pass.clone() {
            Some(p) if !p.is_empty() => p,
            _ => {
                return Ok(PmsResponse {
                    status: "error".into(),
                    message: "pass is required".into(),
                });
            }
        };
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

        self.repo.checkin_repo(&booking).await?;

        Ok(PmsResponse {
            status: "success".into(),
            message: format!("Room {} successfully checked-in", booking.room_number),
        })
    }

    async fn handle_checkout(&self, query: PmsQueryParams) -> Result<PmsResponse> {
        // --- 1️⃣ Input validation ---
        let room = query.room.clone().ok_or_else(|| anyhow!("missing room"))?;

        let booking = Booking {
            room_number: room,
            password: "".into(),
            name: None,
            checkin_date: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            checkout_date: chrono::NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            folio_number: None,
            gtype: None,
        };

        self.repo.checkout_repo(&booking).await?;

        Ok(PmsResponse {
            status: "success".into(),
            message: format!("Room {} successfully checked-out", booking.room_number),
        })
    }
}
