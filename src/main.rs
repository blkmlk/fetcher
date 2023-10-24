use crate::http::server::run_server;
use crate::storage::connection::Connection;
use crate::storage::db::postgres::Client;
use crate::storage::db::mysql;

mod config;
mod storage;
mod http;
mod domain;

#[tokio::main]
async fn main() {
    let config_path = "./config.json";
    run_server("127.0.0.1:8099", config_path).await.unwrap();
}