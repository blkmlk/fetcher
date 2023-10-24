use std::cell::RefCell;
use std::ops::Index;
use mysql_async::prelude::Queryable;
use mysql_async::Value;
use crate::storage::connection;
use crate::storage::connection::{Connection, Row};

pub struct Client {
    pool: mysql_async::Pool
}

impl Client {
    pub fn new(url: &str) -> Self {
        let pool = mysql_async::Pool::new(url);

        Self {
            pool,
        }
    }
}

impl Connection for Client {
    fn exec(&self, query: &str) -> connection::ExecResult {
        let query = query.to_string();
        return Box::pin(
            async move {
                let mut conn = self.pool.get_conn().await?;
                let rows: Vec<mysql_async::Row> = conn.query(&query).await?;
                let mut result = vec![];

                for row in rows {
                    let mut columns = vec![];

                    for i in 0..row.len() {
                        let value: String = match row.index(i) {
                            Value::NULL => String::from("null"),
                            Value::Bytes(v) => String::from_utf8_lossy(v).to_string(),
                            Value::Int(v) => v.to_string(),
                            Value::UInt(v) => v.to_string(),
                            Value::Float(v) => v.to_string(),
                            Value::Double(v) => v.to_string(),
                            _ => return Err("unsupported type".into())
                        };

                        let name = String::from_utf8_lossy(row.columns_ref()[i].name_ref()).to_string();

                        columns.push((name, value));
                    }
                    result.push(Row{columns});
                }

                Ok(result)
            }
        );
    }
}

#[cfg(test)]
mod test {
    use crate::storage::db::mysql::Client;
    use mysql::prelude::Queryable;
    use crate::storage::connection::Connection;

    const DB_URL: &str = "mysql://mysql:mysql@localhost:13306/test";

    #[tokio::test]
    async fn exec() {
        drop_data();
        init_data();
        let client = Client::new(DB_URL);

        let rows = client.exec("select id, name, flag from test").await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].columns.len(), 3);
        assert_eq!(rows[0].columns.iter().map(|x| x.0.to_owned()).collect::<Vec<_>>(), vec!["id", "name", "flag"]);
        assert_eq!(rows[0].columns.iter().map(|x| x.1.to_owned()).collect::<Vec<_>>(), vec!["1", "Islam", "1"]);
    }

    fn init_data() {
        let pool = mysql::Pool::new(DB_URL).unwrap();
        let mut conn = pool.get_conn().unwrap();

        conn.exec_drop("create table test(id int PRIMARY KEY, name varchar(255), flag int)", ()).unwrap();
        conn.exec_drop("insert into test (id, name, flag) values (1, 'Islam', 1)",()).unwrap();
    }

    fn drop_data() {
        let pool = mysql::Pool::new(DB_URL).unwrap();
        let mut conn = pool.get_conn().unwrap();
        conn.exec_drop("drop table test", ()).unwrap_or(());
    }
}