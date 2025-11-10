use dotenvy::dotenv;
use infrastructure::database::init_db_pool;
use presentation::routes::router;
use salvo::prelude::*;
use std::env;

mod application;
mod domain;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt().init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    init_db_pool(&database_url)
        .await
        .expect("Failed to init DB Pool");

    let app_port = std::env::var("APP_PORT")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "5800".to_string());

    let bind_addr = format!("0.0.0.0:{}", app_port);

    let acceptor = TcpListener::new(bind_addr).bind().await;
    let router = router();

    println!("{:?}", router);

    Server::new(acceptor).serve(router).await;
}
