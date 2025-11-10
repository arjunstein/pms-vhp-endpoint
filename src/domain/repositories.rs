use crate::domain::entities::Booking;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn checkin_repo(&self, booking: &Booking) -> Result<()>;
    async fn checkout_repo(&self, booking: &Booking) -> Result<()>;
}
