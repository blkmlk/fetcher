use std::error::Error;

pub trait Connection {
    fn exec(& self, query: String) -> Result<Vec<Row>, Box<dyn Error>>;
}

pub struct Row {
    pub(crate) columns: Vec<(String, String)>
}
