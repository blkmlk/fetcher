use std::cell::RefCell;
use std::error::Error;
use postgres::{Column, NoTls};
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
                let value = parse_column(i, &r, &c)?;
                columns.push((c.name().to_string(), value));
            }
            result.push(Row{columns});
        }

        Ok(result)
    }
}

fn parse_column(idx: usize, row: &postgres::Row, col: &Column) -> Result<String, Box<dyn Error>> {
    match col.type_().name() {
        "int4" => {
            let v: i32 = row.get(idx);
            Ok(v.to_string())
        }
        "varchar" => Ok(row.get(idx)),
        "bool" => {
            let v: bool = row.get(idx);
            Ok(v.to_string())
        },
        v => Err(format!("unknown type {}", v).into())
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

        let rows = client.exec("select id, name, flag from test".to_string()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].columns.len(), 3);
        assert_eq!(rows[0].columns.iter().map(|x| x.0.to_owned()).collect::<Vec<_>>(), vec!["id", "name", "flag"]);
        assert_eq!(rows[0].columns.iter().map(|x| x.1.to_owned()).collect::<Vec<_>>(), vec!["1", "Islam", "true"]);
    }

    fn init_data() {
        let mut conn = postgres::Client::connect(DB_URL, NoTls).unwrap();
        conn.execute("create table test(id int PRIMARY KEY, name varchar, flag boolean)", &[]).unwrap();
        conn.execute("insert into test (id, name, flag) values (1, 'Islam', true)", &[]).unwrap();
    }

    fn drop_data() {
        let mut conn = postgres::Client::connect(DB_URL, NoTls).unwrap();
        conn.execute("drop table test", &[]).unwrap();
    }
}
