use crate::config::DatabaseConfig;
use anyhow::Result;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

#[derive(Debug, Clone)]
pub struct DbManager {
    pool: MySqlPool,
}
impl DbManager {
    pub async fn new(config: &DatabaseConfig) -> Result<Self> {
        let database_url = format!(
            "mysql://{}:{}@{}:{}/{}",
            config.username, config.password, config.host, config.port, config.database_name
        );
        let pool = MySqlPoolOptions::new()
            .min_connections(config.min_connections)
            .max_connections(config.max_connections)
            .connect(&database_url)
            .await?;
        Ok(Self { pool })
    }
    pub fn get_pool(&self) -> &MySqlPool {
        &self.pool
    }
    pub async fn close(&self) {
        self.pool.close().await;
        println!("Db Connection Pool is shutting down")
    }
}
