use crate::application::dtos::{PmsQueryParams, PmsResponse};
use crate::application::utils::{
    datetime_utils::{parse_checkin_datetime, parse_checkout_datetime},
    string_utils::{clean_password, get_formatted_name},
};
use crate::domain::{entities::Booking, repositories::BookingRepository};
use anyhow::{Result, anyhow};
use chrono::Local;
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
                Err(anyhow!("invalid mode"))
            }
        }
    }

    async fn handle_checkin(&self, query: PmsQueryParams) -> Result<PmsResponse> {
        // --- 1️⃣ Input validation ---
        let room = match query.room.clone() {
            Some(r) if !r.is_empty() => r,
            _ => return Err(anyhow!("room is required")),
        };

        let pass_raw = match query.pass.clone() {
            Some(p) if !p.is_empty() => p,
            _ => return Err(anyhow!("pass is required")),
        };

        let cidate_str = match &query.cidate {
            Some(s) if !s.trim().is_empty() => s.trim(),
            _ => return Err(anyhow!("cidate is required")),
        };

        let codate_str = match &query.codate {
            Some(s) if !s.trim().is_empty() => s.trim(),
            _ => return Err(anyhow!("codate is required")),
        };

        if self.repo.is_room_active(&room).await? {
            return Err(anyhow!("room {} is in use", room));
        }

        // --- 2️⃣ Parse date/time ---
        let checkin_datetime = parse_checkin_datetime(cidate_str)?;
        let checkout_datetime = parse_checkout_datetime(codate_str, query.cotime.as_deref())?;

        let pass = clean_password(&pass_raw);

        let formatted_name = get_formatted_name(&query.name, &query.pass);

        // --- 3️⃣ Create domain entity Booking ---
        let booking = Booking {
            room_number: room,
            password: pass,
            name: Some(formatted_name.clone()),
            checkin_date: checkin_datetime,
            checkout_date: checkout_datetime,
            folio_number: query.rsvno.clone(),
            gtype: query.gtype,
        };

        self.repo.checkin_repo(&booking).await?;

        Ok(PmsResponse {
            status: "success".into(),
            message: format!("room {} successfully checkin", booking.room_number),
        })
    }

    async fn handle_checkout(&self, query: PmsQueryParams) -> Result<PmsResponse> {
        // --- 1️⃣ Input validation ---
        let room = match query.room.clone() {
            Some(r) if !r.is_empty() => r,
            _ => return Err(anyhow!("room is required")),
        };

        if !self.repo.is_room_active(&room).await? {
            return Err(anyhow!("room {} not found for checkout", room));
        }

        let booking = Booking {
            room_number: room,
            password: "".into(),
            name: None,
            checkin_date: Local::now().naive_local(),
            checkout_date: Local::now().naive_local(),
            folio_number: None,
            gtype: None,
        };

        self.repo.checkout_repo(&booking).await?;

        Ok(PmsResponse {
            status: "success".into(),
            message: format!("room {} successfully checkout", booking.room_number),
        })
    }
}
