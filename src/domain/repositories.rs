use crate::domain::entities::Booking;
use anyhow::Result;
use async_trait::async_trait;
use sqlx::{MySql, Transaction};

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn checkin_repo(&self, booking: &Booking) -> Result<()>;
    async fn checkout_repo(&self, booking: &Booking) -> Result<()>;
    async fn get_cron_hotel_service(
        &self,
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<Vec<(i32, String)>>;
}
