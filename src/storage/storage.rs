use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, RwLock};
use crate::storage::connection::{Connection, ExecResult, Row};
use crate::config::config::Connection as ConfigConnection;

pub struct Storage {
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

    pub async fn exec(&self, conn_t: ConfigConnection, query: &str) -> Result<Vec<Row>, Box<dyn Error>> {
        let mp = self.connections.read().unwrap();
        let conn: &Box<dyn Connection> = mp.get(&conn_t).unwrap();
        conn.exec(query).await
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
        fn exec(&self, query: &str) -> ExecResult {
            Box::pin(
               async move {
                   Ok(vec![])
               }
            )
        }
    }

    #[tokio::test]
    async fn exec_query() {
        let storage = Storage::new();
        storage.add_connection(PostgresSQL, Box::new(MockConnection::new()));

        let res = storage.exec(PostgresSQL, "test query").await;
        assert!(res.is_ok());
    }
}