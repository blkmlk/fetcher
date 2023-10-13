use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub type ExecResult<'a> = Pin<Box<dyn Future<Output=Result<Vec<Row>, Box<dyn Error>>> + 'a>>;

pub trait Connection<> {
    fn exec(&self, query: String) -> ExecResult;
}

pub struct Row {
    pub(crate) columns: Vec<(String, String)>
}
