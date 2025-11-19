use chrono::Local;
use once_cell::sync::OnceCell;
use std::fs::{OpenOptions, create_dir_all};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

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
