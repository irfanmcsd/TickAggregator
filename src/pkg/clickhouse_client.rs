use crate::pkg::dbcontext::entities::{SymbolKlineData, clean_symbol};
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use clickhouse::Client;
use clickhouse_derive::Row;
use log::info;
use serde::Serialize;
use std::sync::Arc;

#[derive(Clone)]
pub struct ClickHouseClient {
    client: Arc<Client>,
}

#[derive(Debug, Serialize, Row)]
struct KlineRow {
    symbol: String,   // matches ClickHouse `symbol String`
    interval: String, // matches `interval String`
    open: f64,        // matches `open Float64`
    high: f64,        // matches `high Float64`
    low: f64,         // matches `low Float64`
    close: f64,       // matches `close Float64`
    volume: f64,      // matches `volume Float64`
    timestamp: DateTime<Utc>,
    trade_count: i64, // matches `trade_count Int64`
}

impl ClickHouseClient {
    pub async fn init(config: &crate::pkg::config::ClickHouseConfig) -> Result<Self> {
        info!("🔧 Initializing ClickHouse client with native-tls feature...");

        // ClickHouse Cloud connection
        let url = &config.url;
        info!("🔗 Connecting to ClickHouse Cloud at: {}", url);

        let client = Client::default()
            .with_url(url)
            .with_user(&config.user)
            .with_password(&config.password)
            .with_database(&config.database);

        info!("🧪 Testing connection...");
        match client.query("SELECT 1").fetch_one::<u8>().await {
            Ok(result) => {
                info!(
                    "✅ ClickHouse Cloud connection successful! Test result: {}",
                    result
                );
                Ok(Self {
                    client: Arc::new(client),
                })
            }
            Err(e) => {
                log::error!("❌ ClickHouse connection failed: {}", e);
                log::error!("💡 Make sure your ClickHouse Cloud credentials are correct");
                log::error!("   URL: {}", url);
                log::error!("   User: default");
                log::error!("   Database: default");
                Err(e.into())
            }
        }
    }

    /// Create kline table if not exists
    pub async fn create_kline_table(&self) -> Result<()> {
        self.client
            .query(
                "CREATE TABLE IF NOT EXISTS kline_data (
                    symbol String,
                    interval String,
                    open Float64,
                    high Float64,
                    low Float64,
                    close Float64,
                    volume Float64,
                    timestamp DateTime,
                    trade_count Int64
                ) ENGINE = MergeTree()
                PARTITION BY toYYYYMM(timestamp)
                ORDER BY (symbol, interval, timestamp)",
            )
            .execute()
            .await?;

        info!("📊 ClickHouse kline_data table created/verified");
        Ok(())
    }

    /// Insert batch of Klines
    async fn insert_klines(&self, klines: &[KlineRow]) -> Result<()> {
        if klines.is_empty() {
            info!("📭 No klines to insert into ClickHouse");
            return Ok(());
        }

        info!(
            "📝 Starting batch insert of {} klines into ClickHouse",
            klines.len()
        );

        // Small test batch first
        let batch_size = std::cmp::min(10, klines.len());
        let test_batch = &klines[0..batch_size];

        match self.insert_batch_with_retry(test_batch).await {
            Ok(_) => {
                info!("✅ Test batch successful, inserting remaining data...");
                if klines.len() > batch_size {
                    let remaining = &klines[batch_size..];
                    self.insert_batch_with_retry(remaining).await?;
                }
                info!(
                    "💾 Successfully saved {} klines into ClickHouse",
                    klines.len()
                );
                Ok(())
            }
            Err(e) => {
                log::error!("❌ Even small test batch failed: {}", e);
                if let Some(first_row) = klines.first() {
                    log::error!("   First row details: {:?}", first_row);
                }
                Err(e)
            }
        }
    }

    /// Insert batch helper with retry
    async fn insert_batch_with_retry(&self, klines: &[KlineRow]) -> Result<()> {
        info!("🔧 Attempting to insert {} rows", klines.len());

        // Use raw SQL for all batches to avoid RowBinary size limit
        let mut query_values = Vec::with_capacity(klines.len());
        for row in klines {
            // Escape single quotes in strings just in case
            let symbol = row.symbol.replace('\'', "''");
            let interval = row.interval.replace('\'', "''");

            query_values.push(format!(
                "('{}','{}',{},{},{},{},{},'{}',{})",
                symbol,
                interval,
                row.open,
                row.high,
                row.low,
                row.close,
                row.volume,
                row.timestamp.format("%Y-%m-%d %H:%M:%S"),
                row.trade_count
            ));
        }

        let query = format!(
            "INSERT INTO kline_data (symbol, interval, open, high, low, close, volume, timestamp, trade_count) VALUES {}",
            query_values.join(",")
        );

        info!("🧪 Executing SQL INSERT batch with {} rows", klines.len());
        self.client.query(&query).execute().await?;
        info!("✅ Batch insert successful");

        Ok(())
    }

    /// Save SymbolKlineData into ClickHouse
    pub async fn save_symbol_klines(&self, data: &[SymbolKlineData], instance: &str) -> Result<()> {
        if data.is_empty() {
            info!("📭 No klines to insert for instance {}", instance);
            return Ok(());
        }

        // Ensure table exists
        if let Err(e) = self.create_kline_table().await {
            log::warn!("⚠️ Failed to ensure table exists: {}", e);
        }

        // Convert & validate
        let rows: Result<Vec<KlineRow>, _> = data
            .iter()
            .map(|k| {
                if k.symbol.is_empty() || k.interval.is_empty() {
                    return Err(anyhow::anyhow!("Invalid data: empty symbol or interval"));
                }
                if !k.open.is_finite()
                    || !k.high.is_finite()
                    || !k.low.is_finite()
                    || !k.close.is_finite()
                    || !k.volume.is_finite()
                {
                    return Err(anyhow::anyhow!(
                        "Invalid data: NaN or infinite values detected"
                    ));
                }
                Ok(KlineRow {
                    symbol: clean_symbol(&k.symbol),
                    interval: k.interval.clone(),
                    open: k.open,
                    high: k.high,
                    low: k.low,
                    close: k.close,
                    volume: k.volume,
                    timestamp: k.open_time,
                    trade_count: k.trade_count, // direct i64
                })
            })
            .collect();

        let validated_rows = rows?;
        info!(
            "🔄 Converting {} SymbolKlineData records for instance: {}",
            validated_rows.len(),
            instance
        );

        self.insert_klines(&validated_rows).await?;
        info!(
            "✅ Successfully saved {} klines for instance: {}",
            validated_rows.len(),
            instance
        );

        Ok(())
    }
    /*pub async fn get_klines(&self, symbol: &str, interval: &str, limit: Option<u32>) -> Result<Vec<KlineRow>> {
        let limit_clause = match limit {
            Some(l) => format!("LIMIT {}", l),
            None => String::new(),
        };

        let query = format!(
            "SELECT symbol, interval, open, high, low, close, volume, timestamp
             FROM kline_data 
             WHERE symbol = ? AND interval = ? 
             ORDER BY timestamp DESC {}",
            limit_clause
        );

        let rows = self.client
            .query(&query)
            .bind(symbol)
            .bind(interval)
            .fetch_all::<KlineRow>()
            .await?;

        info!("📈 Retrieved {} kline records for {}-{}", rows.len(), symbol, interval);
        Ok(rows)
    }

    /// Get latest timestamp for a symbol and interval (useful for incremental updates)
    pub async fn get_latest_timestamp(&self, symbol: &str, interval: &str) -> Result<Option<NaiveDateTime>> {
        let result: Option<NaiveDateTime> = self.client
            .query("SELECT MAX(timestamp) FROM kline_data WHERE symbol = ? AND interval = ?")
            .bind(symbol)
            .bind(interval)
            .fetch_optional()
            .await?;

        Ok(result)
    }*/

    /// Count total records for monitoring
    pub async fn count_klines(&self) -> Result<u64> {
        let count: u64 = self
            .client
            .query("SELECT COUNT(*) FROM kline_data")
            .fetch_one()
            .await?;

        Ok(count)
    }

    /// Health check method
    pub async fn health_check(&self) -> Result<bool> {
        match self.client.query("SELECT 1").fetch_one::<u8>().await {
            Ok(_) => Ok(true),
            Err(e) => {
                log::error!("❌ ClickHouse health check failed: {}", e);
                Ok(false)
            }
        }
    }
}
