use crate::http::server::run_server;

mod config;
mod storage;
mod http;

#[actix_web::main]
async fn main() {
    run_server("127.0.0.1:8099").await.unwrap();
}