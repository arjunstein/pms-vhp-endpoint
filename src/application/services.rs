use crate::application::dtos::{PmsQueryParams, PmsResponse};
use crate::application::errors::ErrorResponse;
use crate::application::utils::{
    datetime_utils::{parse_checkin_datetime, parse_checkout_datetime},
    string_utils::{clean_password, get_formatted_name},
};
use crate::domain::{entities::Booking, repositories::BookingRepository};
use anyhow::Result;
use chrono::Local;
use std::sync::Arc;

pub struct BookingService<R: BookingRepository> {
    repo: Arc<R>,
}

impl<R: BookingRepository> BookingService<R> {
    pub fn new(repo: Arc<R>) -> Self {
        Self { repo }
    }

    pub async fn process(&self, query: PmsQueryParams) -> Result<PmsResponse, ErrorResponse> {
        match query.mode.as_str() {
            "checkin" => self.handle_checkin(query).await,
            "checkout" => self.handle_checkout(query).await,
            "update" => self.handle_update(query).await,
            mode => {
                tracing::error!("invalid mode {}", mode);
                Err(ErrorResponse::Validation(format!("invalid mode {}", mode)))
            }
        }
    }

    async fn handle_checkin(&self, query: PmsQueryParams) -> Result<PmsResponse, ErrorResponse> {
        let room = match query.room.clone() {
            r if !r.trim().is_empty() => r,
            _ => return Err(ErrorResponse::Validation("room is required".into())),
        };

        let pass = match &query.pass {
            Some(p) if !p.trim().is_empty() => Some(clean_password(p)),
            _ => None,
        };

        let checkin_datetime = match &query.cidate {
            Some(s) if !s.trim().is_empty() => parse_checkin_datetime(s.trim())?,
            _ => Local::now().naive_local(),
        };

        let checkout_datetime = match &query.codate {
            Some(s) if !s.trim().is_empty() => {
                parse_checkout_datetime(s.trim(), query.cotime.as_deref())?
            }
            _ => Local::now().naive_local(),
        };

        if self.repo.is_room_active(&room).await? {
            return Err(ErrorResponse::Validation(format!(
                "room {} is in use",
                room
            )));
        }

        let formatted_name = get_formatted_name(&query.name, &query.pass);

        let booking = Booking {
            room_number: room,
            password: pass,
            name: Some(formatted_name.clone()),
            checkin_date: checkin_datetime,
            checkout_date: checkout_datetime,
            folio_number: query.rvno.clone(),
            gtype: query.gtype,
        };

        self.repo.checkin_repo(&booking).await?;

        Ok(PmsResponse {
            status: "success".into(),
            message: format!("room {} successfully checkin", booking.room_number),
        })
    }

    async fn handle_checkout(&self, query: PmsQueryParams) -> Result<PmsResponse, ErrorResponse> {
        let room = match query.room.clone() {
            r if !r.trim().is_empty() => r,
            _ => return Err(ErrorResponse::Validation("room is required".into())),
        };

        if !self.repo.is_room_active(&room).await? {
            return Err(ErrorResponse::NotFound(format!(
                "room {} not found for checkout",
                room
            )));
        }

        let booking = Booking {
            room_number: room,
            password: Some("".to_string()),
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

    async fn handle_update(&self, query: PmsQueryParams) -> Result<PmsResponse, ErrorResponse> {
        let new_room = match query.room.clone() {
            r if !r.trim().is_empty() => r,
            _ => return Err(ErrorResponse::Validation("room is required".into())),
        };

        let old_room_opt = query.oldroom.clone();

        let pass = match &query.pass {
            Some(p) if !p.trim().is_empty() => Some(clean_password(p)),
            _ => None,
        };

        let checkin_datetime = match &query.cidate {
            Some(s) if !s.trim().is_empty() => parse_checkin_datetime(s.trim())?,
            _ => Local::now().naive_local(),
        };

        let checkout_datetime = match &query.codate {
            Some(s) if !s.trim().is_empty() => {
                parse_checkout_datetime(s.trim(), query.cotime.as_deref())?
            }
            _ => Local::now().naive_local(),
        };

        let old_room = old_room_opt.unwrap_or_else(|| new_room.clone());
        let is_change_room = old_room != new_room;

        if !self.repo.is_room_active(&old_room).await? {
            return Err(ErrorResponse::NotFound(format!(
                "room {} not found for update",
                old_room
            )));
        }

        if is_change_room && self.repo.is_room_active(&new_room).await? {
            return Err(ErrorResponse::Validation(format!(
                "target room {} is already in use",
                new_room
            )));
        }

        let formatted_name = get_formatted_name(&query.name, &query.pass);

        let booking = Booking {
            room_number: new_room.clone(),
            password: pass,
            name: Some(formatted_name.clone()),
            checkin_date: checkin_datetime,
            checkout_date: checkout_datetime,
            folio_number: query.rvno.clone(),
            gtype: query.gtype.clone(),
        };

        self.repo.update_repo(&old_room, &booking).await?;

        let msg = if is_change_room {
            format!("room {} successfully updated to {}", old_room, new_room)
        } else {
            format!("room {} successfully updated", new_room)
        };

        Ok(PmsResponse {
            status: "success".into(),
            message: msg,
        })
    }
}
