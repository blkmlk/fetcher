use crate::domain::fetcher::{Error, Fetcher};

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

    pub async fn get_entity(&self, id: &str) -> Result<(), &'static str>{
        self.fetcher.fetch_id(id).await;

        Ok(())
    }
}
