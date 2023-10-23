use crate::domain::fetcher::{Error, Fetcher, Value};

pub struct EntityHandler {
    fetcher: Fetcher
}

impl EntityHandler {
    pub fn new(config_path: &str) -> Result<Self, String> {
        let fetcher = Fetcher::new(config_path).map_err(|e| match e {
            Error::ConfigFileErr(msg) => format!("failed to read config file: {}", msg),
            _ => format!("failed to init fetcher")
        })?;

        Ok(Self {
            fetcher
        })
    }

    pub async fn get_entity(&self, id: &str) -> Result<Vec<(String, Vec<(String, Value)>)>, String>{
        let resp = match self.fetcher.fetch_id(id).await {
            Ok(v) => v,
            Err(e) => return match e {
                Error::ConfigFileErr(msg) => Err(msg),
                Error::ExecErr(msg) => Err(msg),
                Error::InvalidConfig => Err(String::from("invalid config"))
            }
        };

        Ok(resp)
    }
}
