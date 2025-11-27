use tokio::process::Command;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::domain::repositories::BookingRepository;
use crate::infrastructure::database::db_pool;
use crate::infrastructure::repositories::MySqlBookingRepository;

pub async fn start_disconnect_scheduler() -> anyhow::Result<()> {
    let repo = MySqlBookingRepository {
        pool: db_pool().clone(),
    };

    let sched = JobScheduler::new().await?;
    let job = Job::new_async("*/30 * * * * *", move |_uuid, _l| {
        let repo = repo.clone(); // important!
        Box::pin(async move {
            if let Err(e) = run_disconnect(&repo).await {
                error!("DISCONNECT_CRON ERROR | {}", e);
            }
        })
    })?;

    sched.add(job).await?;
    sched.start().await?;
    Ok(())
}

async fn run_disconnect(repo: &dyn BookingRepository) -> Result<(), String> {
    let junks = repo
        .get_pending_disconnects()
        .await
        .map_err(|e| e.to_string())?;

    if junks.is_empty() {
        return Ok(());
    }

    let pool = db_pool();
    let secret_row: Option<(String,)> = sqlx::query_as("SELECT secret FROM nas LIMIT 1")
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    let secret = match secret_row {
        Some((s,)) => s,
        None => return Err("NAS secret not found".into()),
    };

    // 🔹 Loop disconnect process
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
            repo.delete_disconnect_record(&username, &ip)
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
