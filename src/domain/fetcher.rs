use std::fs;
use futures_executor::block_on;
use crate::config::config;
use crate::domain::fetcher::Error::ConfigFileErr;
use crate::storage;
use crate::storage::storage::Storage;

pub enum Error {
    ConfigFileErr(String),
    ExecErr(String)
}

pub struct Fetcher {
    cfg: config::Config,
    storage: Storage
}

impl Fetcher {
    pub fn new(config_path: &str) -> Result<Self, Error> {
        let data = fs::read(config_path).map_err(|e| ConfigFileErr(e.to_string()))?;
        let cfg = config::parse(data.as_slice()).map_err(|e| ConfigFileErr(e.to_string()))?;

        let storage = Storage::new();

        let ps = block_on(storage::db::postgres::Client::new_async("host=localhost port=15432 user=postgres password=postgres dbname=test".to_string()));
        let ms = storage::db::mysql::Client::new("mysql://mysql:mysql@localhost:13306/test".to_string());

        storage.add_connection(config::Connection::PostgresSQL, Box::new(ps));
        storage.add_connection(config::Connection::MySQL, Box::new(ms));

        Ok(Self {
            cfg,
            storage
        })
    }

    pub async fn fetch_id(&self, id: &str) -> Result<(), Error> {

    }
}