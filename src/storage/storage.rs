use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use crate::storage::connection::{Connection, ExecResult, Row};
use crate::config::config::Connection as ConfigConnection;

struct Storage {
    connections: RwLock<HashMap<ConfigConnection, Box<dyn Connection>>>
}

impl Storage {
    pub fn new() -> Self {
        Self {
            connections: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_connection(&self, conn_t: ConfigConnection, conn: Box<dyn Connection>) {
        let mut mp = self.connections.write().unwrap();
        mp.insert(conn_t, conn);
    }

    pub fn exec(&self, conn_t: ConfigConnection, query: String) -> ExecResult {
        return Box::pin(
            async move {
                let mp = self.connections.read().unwrap();
                let conn: &Box<dyn Connection> = mp.get(&conn_t).unwrap();
                conn.exec(query).await
            }
        );
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;
    use futures::join;
    use futures_executor::block_on;
    use futures_util::future::err;
    use futures_util::FutureExt;
    use crate::config::config::Connection::{MySQL, PostgresSQL};
    use crate::storage::connection::{Connection, ExecResult, Row};
    use crate::storage::db;
    use crate::storage::storage::Storage;

    struct MockConnection;
    impl MockConnection {
        pub fn new() -> Self {Self {}}
    }
    impl Connection for MockConnection {
        fn exec(&self, query: String) -> ExecResult {
            Box::pin(
               async move {
                   Ok(vec![])
               }
            )
        }
    }

    #[test]
    fn exec_query() {
        let storage = Storage::new();
        storage.add_connection(PostgresSQL, Box::new(MockConnection::new()));

        let res = block_on(async move {
            storage.exec(PostgresSQL, String::from("test query")).await
        });
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn exec_multiple() {
        let pg_client = db::postgres::Client::new_async(String::from("host=localhost port=15432 user=postgres password=postgres dbname=test")).await;
        let mysql_client = db::mysql::Client::new(String::from("mysql://mysql:mysql@localhost:13306/test"));

        let mut storage = Storage::new();
        storage.add_connection(PostgresSQL, Box::new(pg_client));
        storage.add_connection(MySQL, Box::new(mysql_client));

        let mut fut1 = storage.exec(PostgresSQL, "SELECT pg_sleep(5)".to_string()).fuse();
        let mut fut2 = storage.exec(MySQL, "SELECT sleep(3)".to_string()).fuse();

        futures::select!(
            a = fut1 => a,
            b = fut2 => b,
        );
    }
}