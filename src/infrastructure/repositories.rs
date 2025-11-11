use crate::domain::{entities::Booking, repositories::BookingRepository};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use sqlx::{MySql, MySqlPool, Transaction};

pub struct MySqlBookingRepository {
    pub pool: MySqlPool,
}

#[async_trait]
impl BookingRepository for MySqlBookingRepository {
    async fn checkin_repo(&self, booking: &Booking) -> Result<()> {
        let mut tx: Transaction<'_, MySql> = self.pool.begin().await?;

        let services = self.get_cron_hotel_service().await?;
        let (service_id, _) = services
            .get(0)
            .ok_or_else(|| anyhow!("No active hotel service found"))?;

        // 1) INSERT to hotel_rooms
        sqlx::query!(
            r#"INSERT INTO hotel_rooms (room_number, password, name, guest_name, service_id, folio_number, checkin_date, checkout_date, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')"#,
            booking.room_number,
            booking.password,
            booking.password.clone(),
            booking.password.clone(),
            service_id,
            booking.folio_number.as_deref().unwrap_or(""),
            booking.checkin_date.and_hms_opt(13, 0, 0),
            booking.checkout_date.and_hms_opt(13, 0, 0),
        )
        .execute(&mut *tx)
        .await?;

        // 2) INSERT to radcheck
        sqlx::query!(
            r#"INSERT INTO radcheck (username, attribute, op, value)
             VALUES (?, 'Cleartext-Password', ':=', ?)"#,
            booking.room_number,
            booking.password
        )
        .execute(&mut *tx)
        .await?;

        // 3) INSERT to radusergroup
        let group = match booking.gtype.as_deref() {
            Some("1") => "VIP",
            _ => "REGULAR",
        };

        sqlx::query!(
            r#"
            INSERT INTO radusergroup (username, groupname, priority, user_type)
            VALUES (?, ?, 1, "hotel-room")
            "#,
            booking.room_number,
            group
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn checkout_repo(&self, booking: &Booking) -> Result<()> {
        let mut tx: Transaction<'_, MySql> = self.pool.begin().await?;

        // 1️⃣ Delete from radcheck
        sqlx::query!(
            "DELETE FROM radcheck WHERE username = ?",
            booking.room_number
        )
        .execute(&mut *tx)
        .await?;

        // 2️⃣ Delete from radusergroup
        sqlx::query!(
            "DELETE FROM radusergroup WHERE username = ?",
            booking.room_number
        )
        .execute(&mut *tx)
        .await?;

        // 3️⃣ Delete from hotel_rooms
        sqlx::query!(
            "DELETE FROM hotel_rooms WHERE room_number = ?",
            booking.room_number
        )
        .execute(&mut *tx)
        .await?;

        // Commit transaction
        tx.commit().await?;
        Ok(())
    }

    async fn get_cron_hotel_service(&self) -> Result<Vec<(i32, String)>> {
        let rows = sqlx::query!(
            r#"
        SELECT id, service_name 
        FROM services 
        WHERE cron = 1 AND cron_type = 'hotel'
        "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| (r.id, r.service_name)).collect())
    }
}
