use crate::domain::{entities::Booking, repositories::BookingRepository};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use chrono::Local;
use sqlx::{MySql, MySqlPool, Transaction};

#[derive(Clone)]
pub struct MySqlBookingRepository {
    pub pool: MySqlPool,
}

#[async_trait]
impl BookingRepository for MySqlBookingRepository {
    async fn checkin_repo(&self, booking: &Booking) -> Result<()> {
        let mut tx: Transaction<'_, MySql> = self.pool.begin().await?;

        let services = self.get_cron_hotel_service().await?;
        let (service_id, service_name) = services
            .get(0)
            .ok_or_else(|| anyhow!("No active hotel service found"))?;

        // 1) INSERT to hotel_rooms
        sqlx::query!(
            r#"INSERT INTO hotel_rooms (room_number, password, name, service_id, folio_number, checkin_date, checkout_date, status)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'active')"#,
            booking.room_number,
            booking.password.as_deref().unwrap_or(""),
            booking.name.as_deref().unwrap_or(""),
            service_id,
            booking.folio_number.as_deref().unwrap_or(""),
            booking.checkin_date,
            booking.checkout_date,
        )
        .execute(&mut *tx)
        .await?;

        // 2) INSERT to radcheck
        sqlx::query!(
            r#"INSERT INTO radcheck (username, attribute, op, value)
             VALUES (?, 'Cleartext-Password', ':=', ?)"#,
            booking.room_number,
            booking.password.as_deref().unwrap_or(""),
        )
        .execute(&mut *tx)
        .await?;

        // 3) INSERT to radusergroup
        sqlx::query!(
            r#"
            INSERT INTO radusergroup (username, groupname, priority, user_type)
            VALUES (?, ?, 1, "hotel-room")
            "#,
            booking.room_number,
            service_name
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

        let online_rooms = self.get_online_rooms(&booking.room_number).await?;

        // Insert online rooms into disconnect pool
        if !online_rooms.is_empty() {
            tracing::info!(
                "{} online rooms send to disconnect_pool | {}",
                online_rooms.len(),
                booking.room_number,
            );
            for (username, nas_ip, framed_ip) in &online_rooms {
                self.insert_into_disconnect_pool(&username, &framed_ip, &nas_ip)
                    .await?;
            }
        }

        // Commit transaction
        tx.commit().await?;
        Ok(())
    }

    async fn update_repo(&self, old_room: &str, booking: &Booking) -> Result<()> {
        let mut tx = self.pool.begin().await?;
        let now = Local::now().naive_local();

        // --- Update hotel_rooms ---
        sqlx::query!(
            r#"
            UPDATE hotel_rooms
            SET room_number = ?, password = COALESCE(?, password), name = COALESCE(?, name), checkin_date = ?, checkout_date = ?, updated_at = ?
            WHERE room_number = ?
            "#,
            booking.room_number,
            booking.password,
            booking.name,
            booking.checkin_date,
            booking.checkout_date,
            now,
            old_room
        )
        .execute(&mut *tx)
        .await?;

        // --- Update radcheck ---
        sqlx::query!(
            "UPDATE radcheck SET username = ?, value = COALESCE(?, value) WHERE username = ?",
            booking.room_number,
            booking.password,
            old_room
        )
        .execute(&mut *tx)
        .await?;

        // --- Update radusergroup ---
        sqlx::query!(
            "UPDATE radusergroup SET username = ? WHERE username = ?",
            booking.room_number,
            old_room
        )
        .execute(&mut *tx)
        .await?;

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

    async fn is_room_active(&self, room_number: &str) -> Result<bool> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM hotel_rooms WHERE room_number = ?")
                .bind(room_number)
                .fetch_one(&self.pool)
                .await?;

        Ok(count > 0)
    }

    async fn get_pending_disconnects(&self) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query!(
            r#"
            SELECT username, ip, nas_ip 
            FROM disconnect_pool 
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.username, r.ip, r.nas_ip))
            .collect())
    }

    async fn delete_disconnect_record(&self, username: &str, ip: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM disconnect_pool WHERE username = ? AND ip = ?",
            username,
            ip
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_online_rooms(&self, room: &str) -> Result<Vec<(String, String, String)>> {
        let rows = sqlx::query!(
            r#"
        SELECT username, nasipaddress, framedipaddress
        FROM radacct
        WHERE username = ? 
        AND acctstoptime IS NULL
        "#,
            room
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.username, r.nasipaddress, r.framedipaddress))
            .collect())
    }

    async fn insert_into_disconnect_pool(
        &self,
        username: &str,
        ip: &str,
        nas_ip: &str,
    ) -> Result<()> {
        sqlx::query!(
            r#"
        INSERT INTO disconnect_pool (username, ip, nas_ip)
        VALUES (?, ?, ?)
        "#,
            username,
            ip,
            nas_ip
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
