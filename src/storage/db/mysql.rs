use std::cell::RefCell;
use std::error::Error;
use std::ops::Index;
use mysql::prelude::Queryable;
use mysql::Value;
use crate::storage::connection::{Connection, Row};

struct Client {
    pool: RefCell<mysql::Pool>
}

impl Client {
    pub fn new(url: String) -> Self {
        let pool = mysql::Pool::new(url.as_str()).unwrap();

        Self {
            pool: RefCell::new(pool),
        }
    }
}

impl Connection for Client {
    fn exec(&self, query: String) -> Result<Vec<Row>, Box<dyn Error>> {
        let mut conn = self.pool.borrow().get_conn()?;
        let rows: Vec<mysql::Row> = conn.query(query)?;
        let mut result = vec![];

        for row in rows {
            let mut columns = vec![];

            for i in 0..row.len() {
                let value: String = match row.index(i) {
                    Value::NULL => String::from("null"),
                    Value::Bytes(v) => String::from(v),
                    Value::Int(v) => v.to_string(),
                    Value::UInt(v) => v.to_string(),
                    Value::Float(v) => v.to_string(),
                    Value::Double(v) => v.to_string(),
                    _ => return Err("unsupported type".into())
                };

                let name = String::from(row.columns_ref()[i].name_ref());

                columns.push((name, value));
            }
            result.push(Row{columns});
        }

        Ok(result)
    }
}

#[cfg(test)]
mod test {
    use crate::storage::db::mysql::Client;
    use mysql::prelude::Queryable;
    use crate::storage::connection::Connection;

    const DB_URL: &str = "";

    #[test]
    fn exec() {
        drop_data();
        init_data();
        let client = Client::new(DB_URL.to_string());

        let rows = client.exec("select id, name, flag from test".to_string()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].columns.len(), 3);
        assert_eq!(rows[0].columns.iter().map(|x| x.0.to_owned()).collect::<Vec<_>>(), vec!["id", "name", "flag"]);
        assert_eq!(rows[0].columns.iter().map(|x| x.1.to_owned()).collect::<Vec<_>>(), vec!["1", "Islam", "true"]);
    }

    fn init_data() {
        let pool = mysql::Pool::new(DB_URL).unwrap();
        let mut conn = pool.get_conn().unwrap();

        conn.exec("create table test(id int PRIMARY KEY, name varchar, flag boolean)", &[]).unwrap();
        conn.exec("insert into test (id, name, flag) values (1, 'Islam', true)", &[]).unwrap();
    }

    fn drop_data() {
        let pool = mysql::Pool::new(DB_URL).unwrap();
        let mut conn = pool.get_conn().unwrap();
        conn.exec("drop table test", &[]).unwrap();
    }
}
