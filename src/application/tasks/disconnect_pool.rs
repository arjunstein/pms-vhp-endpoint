use sqlx::{query, query_as};
use tokio::process::Command;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::infrastructure::database::db_pool;

pub async fn start_disconnect_scheduler() -> anyhow::Result<()> {
    let sched = JobScheduler::new().await?;

    let job = Job::new_async("0 */30 * * * *", |_uuid, _l| {
        Box::pin(async move {
            if let Err(e) = run_disconnect().await {
                error!("DISCONNECT_CRON ERROR | {}", e);
            }
        })
    })?;

    sched.add(job).await?;
    sched.start().await?;

    Ok(())
}

async fn run_disconnect() -> Result<(), String> {
    let pool = db_pool();

    let junks: Vec<(String, String, String)> =
        query_as("SELECT username, ip, nas_ip FROM disconnect_pool")
            .fetch_all(pool)
            .await
            .map_err(|e| e.to_string())?;

    if junks.is_empty() {
        return Ok(());
    }

    let secret_row: Option<(String,)> = query_as("SELECT secret FROM nas LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    let secret = match secret_row {
        Some((s,)) => s,
        None => return Err("NAS secret not found".into()),
    };

    // loop user
    for (username, ip, nas_ip) in junks {
        let cmd_str = format!(
            r#"echo "Framed-IP-Address={}" | /usr/bin/radclient {}:1700 disconnect {}"#,
            ip, nas_ip, secret
        );

        let output = Command::new("sh")
            .arg("-c")
            .arg(&cmd_str)
            .output()
            .await
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            query("DELETE FROM disconnect_pool WHERE username = ? AND ip = ?")
                .bind(&username)
                .bind(&ip)
                .execute(pool)
                .await
                .map_err(|e| e.to_string())?;

            info!(
                "DISCONNECT_CRON SUCCESS | Disconnected {} ({})",
                username, ip
            );
        } else {
            error!(
                "DISCONNECT_CRON WARN | radclient failed for {} ({}): {:?}",
                username, ip, output
            );
        }
    }

    Ok(())
}
