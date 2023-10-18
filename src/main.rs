use crate::http::server::run_server;

mod config;
mod storage;
mod http;

#[tokio::main]
async fn main() {
    let config_path = "./config.json";
    run_server("127.0.0.1:8099", config_path).await.unwrap();
}