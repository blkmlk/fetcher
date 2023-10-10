use std::collections::HashMap;
use std::error::Error;
use std::sync::RwLock;
use crate::storage::connection::{Connection, Row};
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

    pub fn exec(&self, conn_t: ConfigConnection, query: String) -> Result<Vec<Row>, Box<dyn Error>> {
        let mp = self.connections.read().unwrap();
        let conn: &Box<dyn Connection> = mp.get(&conn_t).unwrap();

        conn.exec(query)
    }
}

#[cfg(test)]
mod test {
    use std::error::Error;
    use crate::config::config::Connection::PostgresSQL;
    use crate::storage::connection::{Connection, Row};
    use crate::storage::storage::Storage;

    struct MockConnection;
    impl MockConnection {
        pub fn new() -> Self {Self {}}
    }
    impl Connection for MockConnection {
        fn exec(& self, query: String) -> Result<Vec<Row>, Box<dyn Error>> {
            Ok(vec![])
        }
    }

    #[test]
    fn exec_query() {
        let storage = Storage::new();
        storage.add_connection(PostgresSQL, Box::new(MockConnection::new()));

        assert!(storage.exec(PostgresSQL, String::from("test query")).is_ok());
    }
}