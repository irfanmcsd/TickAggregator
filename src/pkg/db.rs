use crate::pkg::config::DatabaseConfig;
use anyhow::Result;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::env;
use log::{info, warn, error};

pub struct DB {
    pub pool: PgPool,
}

impl DB {
    pub async fn init(database_config: &DatabaseConfig) -> Result<Self, anyhow::Error> {
        if database_config.provider.to_lowercase() != "postgresql" {
            panic!("Unsupported DB provider: {}", database_config.provider);
        }

        // Load connection info from environment variables (like your Go code)
        let host = env::var("DB_HOST").unwrap_or_else(|_| "".into());
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".into());
        let user = env::var("DB_USER").unwrap_or_else(|_| "".into());
        let password = env::var("DB_PASSWORD").unwrap_or_else(|_| "".into());
        let dbname = env::var("DB_NAME").unwrap_or_else(|_| "".into());

        if host.is_empty() || user.is_empty() || password.is_empty() || dbname.is_empty() {
            error!("Missing one or more required DB environment variables: DB_HOST, DB_PORT, DB_USER, DB_PASSWORD, DB_NAME");
            panic!("Missing DB environment variables");
        }

        // Build connection string similar to Go's format:
        // "host=... port=... user=... password=... dbname=... sslmode=disable"
        let connection_string = format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, dbname
        );

        info!("Connecting to PostgreSQL at {}", host);

        // Create connection pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&connection_string)
            .await?;

        // Test connection
        pool.acquire().await?;

        info!("✅ PostgreSQL connected");

        Ok(Self { pool })
    }
}