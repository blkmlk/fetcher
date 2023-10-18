use std::io;
use std::os::macos::raw::stat;
use std::sync::{Arc, Mutex};
use actix_web::{App, HttpServer, web};
use mysql_async::prelude::Query;
use crate::http::factory::route_factory;
use crate::http::handlers::EntityHandler;

pub struct State {
    pub entity_handler: Mutex<EntityHandler>
}

pub async fn run_server(addr: &str, config_path: &str) -> Result<(), io::Error> {
    let state = web::Data::new(State{
        entity_handler: Mutex::new(EntityHandler::new(config_path)),
    });

    HttpServer::new(move || {
        let app = App::new().
            app_data(state.clone()).
            configure(route_factory);
        app
    }).bind(addr)?.run().await
}