use std::cell::RefCell;
use std::error::Error;
use std::future::Future;
use futures_executor::block_on;
use futures_util::task::SpawnExt;
use tokio::task::JoinHandle;
use tokio_postgres::GenericClient;
use crate::storage::connection;
use crate::storage::connection::{Connection, Row};

pub struct Client {
    client: tokio_postgres::Client,
    sp: JoinHandle<()>,
}

impl Drop for Client {
    fn drop(&mut self) {
        self.close();
    }
}

impl Client {
    pub async fn new(url: &str) -> Self {
        let (client, conn) = tokio_postgres::connect(url, tokio_postgres::NoTls).await.unwrap();

        let t = tokio::spawn(async move{
            if let Err(e) = conn.await {
                panic!("{}", e);
            }
        });

        Self {
            client,
            sp: t,
        }
    }

    pub fn close(&self) {
        self.sp.abort();
    }
}

impl Connection for Client {
    fn exec(&self, query: &str) -> connection::ExecResult {
        let query = query.to_string();
        return Box::pin(
            async move {
                let resp = self.client.query(&query, &[]).await?;
                let mut result = vec![];

                for row in resp {
                    let mut columns = vec![];
                    for col in row.columns() {
                        let value = parse_column_value(&row, &col)?;
                        columns.push((col.name().to_string(), value));
                    }
                    result.push(Row{columns});
                }

                Ok(result)
            }
        );
    }
}

fn parse_column_value(row: &tokio_postgres::Row, col: &tokio_postgres::Column) -> Result<String, Box<dyn Error>> {
    match col.type_().name() {
        "void" => Ok(String::default()),
        "int4" => {
            let v: i32 = row.get(col.name());
            Ok(v.to_string())
        }
        "varchar" => Ok(row.get(col.name())),
        "bool" => {
            let v: bool = row.get(col.name());
            Ok(v.to_string())
        },
        v => Err(format!("unknown type {}", v).into())
    }
}

#[cfg(test)]
mod test {
    use futures_executor::block_on;
    use postgres::NoTls;
    use crate::storage::connection::Connection;
    use crate::storage::db::postgres::Client;

    const DB_URL: &str = "host=localhost port=15432 user=postgres password=postgres dbname=test";

    #[tokio::test]
    async fn exec() {
        drop_data().await;
        init_data().await;
        let client = block_on(Client::new(DB_URL));

        // let rows = client.exec("select id, name, flag from test").await.unwrap();
        // assert_eq!(rows.len(), 1);
        // assert_eq!(rows[0].columns.len(), 3);
        // assert_eq!(rows[0].columns.iter().map(|x| x.0.to_owned()).collect::<Vec<_>>(), vec!["id", "name", "flag"]);
        // assert_eq!(rows[0].columns.iter().map(|x| x.1.to_owned()).collect::<Vec<_>>(), vec!["1", "Islam", "true"]);
    }

    async fn init_data() {
        let (mut conn, conn2) = tokio_postgres::connect(DB_URL, NoTls).await.unwrap();
        tokio::spawn(async move{
            if let Err(e) = conn2.await {
                panic!("{}", e);
            }
        });
        conn.execute("create table test(id int PRIMARY KEY, name varchar, flag boolean)", &[]).await.unwrap();
        conn.execute("insert into test (id, name, flag) values (1, 'Islam', true)", &[]).await.unwrap();
    }

    async fn drop_data() {
        let (mut conn, conn2) = tokio_postgres::connect(DB_URL, NoTls).await.unwrap();
        tokio::spawn(async move{
            if let Err(e) = conn2.await {
                panic!("{}", e);
            }
        });
        conn.execute("drop table test", &[]).await.unwrap_or(0);
    }
}
