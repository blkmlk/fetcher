use std::collections::HashMap;
use std::fs;
use futures_executor::block_on;
use futures_util::future::join_all;
use crate::config::config;
use crate::domain::fetcher::Error::{ConfigFileErr, ExecErr, InvalidConfig};
use crate::storage;
use crate::storage::connection::{ExecResult, Row};
use crate::storage::storage::Storage;
use std::error::Error as StdError;
use std::ops::Deref;
use serde::{Serialize, Serializer};
use serde::ser::SerializeSeq;
use serde_json::json;
use crate::config::config::{ExpectedRows, Property};

pub enum Error {
    ConfigFileErr(String),
    ExecErr(String),
    InvalidConfig,
}

pub struct Fetcher {
    cfg: config::Config,
    storage: Storage
}

#[derive(Clone)]
pub enum Value {
    String(String),
    Array(Vec<String>)
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

    pub async fn fetch_id(&self, id: &str) -> Result<Vec<(String, Vec<(String, Value)>)>, Error> {
        let attrs = self.cfg.attr_groups.iter().collect::<Vec<_>>();
        let mut futs = vec![];

        for (i, &&ref attr) in attrs.iter().enumerate() {
            for (j, group) in attr.1.iter().enumerate() {
                let query = group.query.replace("__PID__", id);
                futs.push(async move {
                    let resp = self.storage.exec(group.conn.clone(), query).await;
                    (i, j, resp)
                })
            }
        }

        let results: Vec<(usize, usize, Result<Vec<Row>, Box<dyn StdError>>)> = join_all(futs).await;

        let mut mapped: HashMap<String, Vec<(String,Value)>> = HashMap::new();

        for res in results {
            let &&ref attr = attrs.get(res.0).expect("unknown attribute");
            let &ref group = attr.1.get(res.1).expect("unknown group");

            let rows = res.2.map_err(|e| ExecErr(e.to_string()))?;

            let rows_iter = if group.exp_rows == ExpectedRows::Single {
                rows.iter().take(1)
            } else {
                if group.select_attrs.len() != 1 {
                    return Err(InvalidConfig)
                }
                rows.iter().take(rows.len())
            };

            let mut attr_values = mapped.entry(attr.0.to_string()).or_insert(Vec::new());

            let mut values = vec![];
            for row in rows_iter {
                for (col_k, col_v) in row.columns.iter() {
                    let name = group.select_attrs.iter().find_map(|(k,v)| {
                        if k == col_k {
                            if let Some(convert) = v.iter().find_map(|vv| {
                                if let Property::ConvertName(cv) = vv {
                                    return Some(cv)
                                }
                                None
                            }) {
                                return Some(convert)
                            }
                            return Some(k)
                        }

                        None
                    });

                    match name {
                        None => continue,
                        Some(vv) => values.push((vv.to_string(), Value::String(col_v.to_string())))
                    }
                }
            }

            if group.exp_rows == ExpectedRows::Multiple && values.len() > 0{
                let &ref val = values.get(0).unwrap();
                let array = values.iter().map(|e| {
                    if let Value::String(v) = e.1.clone() {
                        return v.to_string()
                    }
                    String::default()
                }).collect::<Vec<String>>();
                values = vec![(val.0.to_string(), Value::Array(array))];
            }

            attr_values.append(&mut values);
        }

        Ok(Vec::from_iter(mapped.into_iter()))
    }
}