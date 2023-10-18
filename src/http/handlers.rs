use std::fs;
use std::future::Future;
use std::io::{BufReader, Bytes, Read};
use std::process::Output;
use futures_executor::block_on;
use futures::{join};
use futures_util::{future, FutureExt};
use crate::config::config;
use crate::config::config::{Config};
use crate::storage;
use crate::storage::connection::{ExecResult, Row};
use crate::storage::storage::Storage;

pub struct EntityHandler {
    storage: Storage,
    config: Config,
}

impl EntityHandler {
    pub fn new(config_path: &str) -> Self {
        let data = fs::read(config_path).unwrap();
        let config = config::parse(data.as_slice()).unwrap();
        let storage = Storage::new();

        let ps = block_on(storage::db::postgres::Client::new_async("host=localhost port=15432 user=postgres password=postgres dbname=test".to_string()));

        storage.add_connection(config::Connection::PostgresSQL, Box::new(ps));

        Self {
            storage,
            config,
        }
    }

    pub async fn add_entity(&mut self, val: i64) -> i64 {
        let f1 = self.storage.exec(config::Connection::PostgresSQL, format!("SELECT {}", val).to_string());
        let f2 = self.storage.exec(config::Connection::PostgresSQL, format!("SELECT {}", val).to_string());
        future::join_all(vec![f1, f2]).await;
        0
    }
}
