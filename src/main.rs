use crate::http::server::run_server;
use crate::storage::connection::Connection;
use crate::storage::db::postgres::Client;
use crate::storage::db::mysql;

mod config;
mod storage;
mod http;
mod domain;

#[tokio::main]
async fn main() {
    init_db().await;

    let config_path = "./config.json";
    run_server("127.0.0.1:8099", config_path).await.unwrap();
}

async fn init_db() {
    {
        let pg = Client::new_async(String::from("host=localhost port=15432 user=postgres password=postgres dbname=test")).await;
        pg.exec(String::from("DROP TABLE users")).await.unwrap_or(vec![]);
        pg.exec(String::from("CREATE TABLE users (id int, username varchar)")).await.unwrap();
        pg.exec(String::from("INSERT INTO users(id, username) VALUES(1, 'Islam')")).await.unwrap();
    }

    {
        let ms = mysql::Client::new(String::from("mysql://mysql:mysql@localhost:13306/test"));
        ms.exec(String::from("DROP TABLE orgs")).await.unwrap_or(vec![]);
        ms.exec(String::from("CREATE TABLE orgs (user_id int, username varchar(200))")).await.unwrap();
        ms.exec(String::from("INSERT INTO orgs(user_id, username) VALUES(1, 'suka')")).await.unwrap();
    }
}