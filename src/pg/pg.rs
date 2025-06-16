use deadpool_postgres::{Config, Manager, ManagerConfig, Pool, RecyclingMethod, Runtime};
use serde::de::StdError;
use std::io::Error;
use std::num::ParseIntError;
use std::{env, fs};
use tokio_postgres::{Client, NoTls};
use tracing::{error, info};

const WORKERS: usize = 16;
const ITERATIONS: usize = 1000;
pub type PgClient = deadpool::managed::Object<Manager>;

pub struct PgConnect {
    host: String,
    user: String,
    password: String,
    dbname: String,
    port: u16,
}
impl PgConnect {
    pub fn host(&mut self, host: String) -> &mut Self {
        self.host = host;
        self
    }
    pub fn user(&mut self, user: String) -> &mut Self {
        self.user = user;
        self
    }
    pub fn password(&mut self, password: String) -> &mut Self {
        self.password = password;
        self
    }
    pub fn dbname(&mut self, dbname: String) -> &mut Self {
        self.dbname = dbname;
        self
    }

    pub fn port(&mut self, port: String) -> Result<&mut Self, ParseIntError> {
        let port: u16 = port.parse()?;
        self.port = port;
        Ok(self)
    }

    pub async fn connect(&self) -> Client {
        let conn_str = format!(
            "host={} user={} password={} dbname={} port={}",
            self.host, self.user, self.password, self.dbname, self.port
        );
        match tokio_postgres::connect(&conn_str, NoTls).await {
            Ok((client, connection)) => {
                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        panic!("Error on connection to db {:?}", e);
                    } else {
                        info!("Successfully connected to postgres conn_str {}", conn_str);
                    }
                });
                client
            }
            Err(e) => {
                panic!("Error connecting to db {:?}", e);
            }
        }
    }
    pub async fn run_migrations(client: &Client) -> Result<(), Box<dyn StdError>> {
        let paths = fs::read_dir("./migrations")?;
        for file in paths {
            let file_name = file?.path();
            let sql = std::fs::read_to_string(file_name)?;
            client.batch_execute(&sql).await?;
            info!("Executed migration {sql}");
        }
        Ok(())
    }

    pub fn create_pool(&self) -> Pool {
        let mut cfg = Config::new();
        cfg.host = Some(self.host.clone());
        cfg.port = Some(self.port);
        cfg.password = Some(self.password.clone());
        cfg.user = Some(self.user.clone());
        cfg.dbname = Some(self.dbname.clone());
        cfg.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        });
        cfg.create_pool(Some(Runtime::Tokio1), NoTls)
            .expect("Failed to create pool")
    }

    pub fn from_env() -> Result<Self, Box<dyn StdError>> {
        let mut pg = Self {
            dbname: env::var("PG_DB")?,
            password: env::var("PG_PASS")?,
            host: env::var("PG_HOST")?,
            user: env::var("PG_USER")?,
            port: 0,
        };
        let port = env::var("PG_PORT")?;
        pg.port(port)?;
        Ok(pg)
    }

    pub fn create_pool_from_env() -> Result<Pool, Box<dyn StdError>> {
        let pool = Self::from_env()?;
        Ok(pool.create_pool())
    }
}
