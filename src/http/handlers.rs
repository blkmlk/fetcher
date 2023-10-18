use std::error::Error;
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
        let ms = storage::db::mysql::Client::new("mysql://mysql:mysql@localhost:13306/test".to_string());

        storage.add_connection(config::Connection::PostgresSQL, Box::new(ps));
        storage.add_connection(config::Connection::MySQL, Box::new(ms));

        Self {
            storage,
            config,
        }
    }

    pub async fn get_entity(&mut self, id: &str) -> Result<Vec<Row>, Box<dyn Error>> {
        let mut result = vec![];
        for gr in self.config.attr_groups.iter() {
            let mut futs = vec![];
            for v in gr.1.iter() {
                let query = v.query.replace("__PID__", id);
                let f = self.storage.exec(v.conn.clone(), query);
                futs.push(f);
            }

            let results = future::join_all(futs).await;

            for r in results {
                match r {
                    Ok(v) => result.extend(v),
                    Err(e) => return Err(e)
                }
            }
        }

        Ok(result)
    }
}
