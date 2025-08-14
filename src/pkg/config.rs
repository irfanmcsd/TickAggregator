use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::{fs, path::Path, sync::Arc};

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregatorSettings {
    #[serde(rename = "EnableJitter")]
    pub enable_jitter: bool,
    #[serde(rename = "JitterMaxMillis")]
    pub jitter_max_millis: i32,
    #[serde(rename = "EnableBatchStats")]
    pub enable_batch_stats: bool,
    #[serde(rename = "PersistRawTicks")]
    pub persist_raw_ticks: PersistRawTicks,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PersistRawTicks {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "Output")]
    pub output: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StreamingConfig {
    #[serde(rename = "Enabled")]
    pub enabled: bool,
    #[serde(rename = "Provider")]
    pub provider: String,
    #[serde(rename = "Redis")]
    pub redis: RedisConfig,
    #[serde(rename = "Kafka")]
    pub kafka: KafkaConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RedisConfig {
    #[serde(rename = "Address")]
    pub address: String,
    #[serde(rename = "Stream")]
    pub stream: String,
    #[serde(rename = "Password")]
    pub password: Option<String>,
    #[serde(rename = "DB")]
    pub db: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct KafkaConfig {
    #[serde(rename = "Brokers")]
    pub brokers: Vec<String>,
    #[serde(rename = "Topic")]
    pub topic: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    #[serde(rename = "provider")]
    pub provider: String,
    #[serde(rename = "connectionString")]
    pub connection_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(rename = "exchange")]
    pub exchange: String,
    #[serde(rename = "instance")]
    pub instance: String,
    #[serde(rename = "RefreshSeconds")]
    pub refresh_seconds: i32,
    #[serde(rename = "symbol")]
    pub symbols: Vec<String>,
    #[serde(rename = "blacklisted_symbols")]
    pub blacklisted_symbols: Vec<String>,
    #[serde(rename = "Aggregator")]
    pub aggregator: AggregatorSettings,
    #[serde(rename = "Streaming")]
    pub streaming: StreamingConfig,
    #[serde(rename = "Debug")]
    pub debug: bool,
    #[serde(rename = "database")]
    pub database: DatabaseConfig,
}

/// Global, thread-safe settings shared across the app.
/// `Arc` allows cheap cloning of the reference for multi-thread usage.
pub static SETTINGS: Lazy<Arc<AppSettings>> = Lazy::new(|| {
    Arc::new(load_config("appsettings.yaml"))
});


fn load_config<P: AsRef<Path>>(path: P) -> AppSettings {
    let data = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read config file {:?}: {}", path.as_ref(), e));

    let settings: AppSettings = serde_yaml::from_str(&data)
        .unwrap_or_else(|e| panic!("Failed to parse config: {}", e));

    println!("✅ App settings loaded from {:?}", path.as_ref());
    settings
}