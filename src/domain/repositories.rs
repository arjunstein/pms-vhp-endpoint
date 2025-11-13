use crate::domain::entities::Booking;
use anyhow::Result;
use async_trait::async_trait;
#[async_trait]
pub trait BookingRepository: Send + Sync {
    async fn checkin_repo(&self, booking: &Booking) -> Result<()>;
    async fn checkout_repo(&self, booking: &Booking) -> Result<()>;
    async fn update_repo(&self, old_room: &str, booking: &Booking) -> Result<()>;
    async fn get_cron_hotel_service(&self) -> Result<Vec<(i32, String)>>;
    async fn is_room_active(&self, room_number: &str) -> Result<bool>;
}
