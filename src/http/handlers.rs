use std::fs;
use std::io::{BufReader, Bytes, Read};
use crate::config::config;
use crate::config::config::{Config};

pub struct EntityHandler {
    config: Config,
}

impl EntityHandler {
    pub fn new(config_path: &str) -> Self {
        let data = fs::read(config_path).unwrap();
        let config = config::parse(data.as_slice()).unwrap();

        Self {
            config,
        }
    }

    pub fn add_entity(&mut self, val: i64) -> i64 {
        0
    }
}
