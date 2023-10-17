use std::io;
use actix_web::{App, HttpServer};
use mysql_async::prelude::Query;
use crate::http::factory::route_factory;

pub async fn run_server(addr: &str) -> Result<(), io::Error> {
    HttpServer::new(|| {
        let app = App::new().configure(route_factory);
        app
    }).bind(addr)?.run().await
}