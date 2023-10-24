use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub type ExecResult<'a> = Pin<Box<dyn Future<Output=Result<Vec<Row>, Box<dyn Error>>> + 'a>>;

pub trait Connection: Send {
    fn exec(&self, query: &str) -> ExecResult;
}

pub struct Row {
    pub columns: Vec<(String, String)>
}