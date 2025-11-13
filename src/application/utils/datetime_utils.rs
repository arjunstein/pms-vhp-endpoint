use anyhow::{Result, anyhow};
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime};

/// Parse string date check-in into `NaiveDateTime`.
/// Accepts format like `20/11/2025` or `20/11/2025 14:30:00`.
pub fn parse_checkin_datetime(cidate_str: &str) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(cidate_str, "%d/%m/%Y %H:%M:%S")
        .or_else(|_| {
            NaiveDate::parse_from_str(cidate_str, "%d/%m/%Y").map(|d| {
                let now_time = Local::now().naive_local().time();
                d.and_time(now_time)
            })
        })
        .map_err(|_| anyhow!("invalid checkin date format"))
}

/// Parse string date checkout into `NaiveDateTime` (combined date + time).
/// Default checkout time: `13:00:00` if `cotime` is not provided.
pub fn parse_checkout_datetime(
    codate_str: &str,
    cotime_str: Option<&str>,
) -> Result<NaiveDateTime> {
    let check_out_date = NaiveDate::parse_from_str(codate_str, "%d/%m/%Y")
        .map_err(|_| anyhow!("invalid checkout date format"))?;

    let check_out_time = match cotime_str {
        Some(t) if !t.trim().is_empty() => NaiveTime::parse_from_str(t.trim(), "%H:%M:%S")
            .map_err(|_| anyhow!("invalid checkout time format"))?,
        _ => NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
    };

    Ok(check_out_date.and_time(check_out_time))
}
