use chrono::Local;
use once_cell::sync::OnceCell;
use std::fs::{OpenOptions, create_dir_all};
use tracing::info;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::application::dtos::PmsQueryParams;

static FILE_GUARD: OnceCell<WorkerGuard> = OnceCell::new();

struct LocalTimer;

impl fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut fmt::format::Writer) -> std::fmt::Result {
        let now = Local::now(); // pakai WIB otomatis jika OS sudah timezone Asia/Jakarta
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S"))
    }
}

pub fn init_logger() {
    let _ = create_dir_all("logs");

    let date = Local::now().format("%Y-%m-%d");
    let file_path = format!("logs/apilog_{}.log", date);

    let file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(file_path)
        .unwrap();

    let (writer, guard) = tracing_appender::non_blocking(file);

    let _ = FILE_GUARD.set(guard);

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_timer(LocalTimer)
        .with_target(false);
    let file_layer = fmt::layer()
        .with_ansi(false)
        .with_writer(writer)
        .with_target(false)
        .with_timer(LocalTimer);

    let env_filter = EnvFilter::new("info");

    tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer)
        .with(env_filter)
        .init();
}

pub fn log_success(query: &PmsQueryParams, room: &str, pass: &Option<String>) {
    let mut log_params: Vec<String> = Vec::new();

    log_params.push(format!("mode => {}", query.mode));
    log_params.push(format!("room => {}", room));

    // only push parameters that are sent
    if let Some(p) = pass {
        log_params.push(format!("pass => {}", p));
    }
    if let Some(cidate) = &query.cidate {
        log_params.push(format!("cidate => {}", cidate));
    }
    if let Some(codate) = &query.codate {
        log_params.push(format!("codate => {}", codate));
    }
    if let Some(cotime) = &query.cotime {
        log_params.push(format!("cotime => {}", cotime));
    }
    if let Some(oldroom) = &query.oldroom {
        log_params.push(format!("oldroom => {}", oldroom));
    }
    if let Some(name) = &query.name {
        log_params.push(format!("name => {}", name));
    }
    if let Some(rvno) = &query.rvno {
        log_params.push(format!("rvno => {}", rvno));
    }
    if let Some(gtype) = &query.gtype {
        log_params.push(format!("gtype => {}", gtype));
    }

    let params_log = log_params.join(", ");
    info!("{} success sending params({})", query.mode, params_log);
}
