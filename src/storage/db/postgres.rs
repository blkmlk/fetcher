use std::cell::RefCell;
use std::error::Error;
use postgres::NoTls;
use crate::storage::connection::{Connection, Row};

struct Client {
    client: RefCell<postgres::Client>
}

impl Client {
    pub fn new(url: String) -> Self {
        let client = postgres::Client::connect(url.as_str(), NoTls).unwrap();

        Self { client: RefCell::new(client) }
    }
}

impl Connection for Client {
    fn exec(&self, query: String) -> Result<Vec<Row>, Box<dyn Error>> {
        let resp = self.client.borrow_mut().query(&query, &[])?;
        let mut result = vec![];

        for r in resp {
            let mut columns = vec![];
            for (i, c) in r.columns().iter().enumerate() {
                let value: String = match c.type_().name() {
                    "int4" => {
                        let v: i32 = r.get(i);
                        v.to_string()
                    }
                    "varchar" => r.get(i),
                    v => {
                        println!("{}", v);
                        "".to_string()
                    }
                };
                columns.push((c.name().to_string(), value));
            }
            result.push(Row{columns});
        }

        Ok(result)
    }
}

mod test {
    use postgres::NoTls;
    use crate::storage::connection::Connection;
    use crate::storage::db::postgres::Client;

    const DB_URL: &str = "host=localhost port=15432 user=postgres password=postgres dbname=test";

    #[test]
    fn exec() {
        drop_data();
        init_data();
        let client = Client::new(DB_URL.to_string());

        let rows = client.exec("select id, name from test".to_string()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].columns.len(), 2);
    }

    fn init_data() {
        let mut conn = postgres::Client::connect(DB_URL, NoTls).unwrap();
        conn.execute("create table test(id int PRIMARY KEY, name varchar)", &[]).unwrap();
        conn.execute("insert into test (id, name) values (1, 'Islam')", &[]).unwrap();
    }

    fn drop_data() {
        let mut conn = postgres::Client::connect(DB_URL, NoTls).unwrap();
        conn.execute("drop table test", &[]).unwrap();
    }
}
