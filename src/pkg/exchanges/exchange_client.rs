use anyhow::{Result, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::pkg::exchanges::binance;
use crate::pkg::exchanges::bitget::{self};
use crate::pkg::exchanges::bybit::{self};
use crate::pkg::exchanges::okx::{self};

#[derive(Debug, Clone)]
pub struct TickerInfo {
    pub symbol: String,
    pub last_price: String,
    pub high_24h: Option<String>,
    pub low_24h: Option<String>,
    pub vol_24h: Option<String>,
    pub change_24h: Option<String>,
    pub exchange: String,
    pub timestamp: u128, // millis since epoch
}

impl TickerInfo {
    fn now_timestamp() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
    }
}

pub async fn core_futures_all_tickers(exchange: &str) -> Result<Vec<TickerInfo>> {
    let ex = exchange.to_lowercase();
    match ex.as_str() {
        "binance" => {
            // Create BinanceApi instance (example, adjust if your struct needs config)
            let api = binance::binance_api::BinanceApi::new();

            // Call the async method on the instance
            let data = api.get_all_tickers().await?;
            let result = data
                .into_iter()
                .map(|t| TickerInfo {
                    symbol: t.symbol,
                    last_price: t.last_price,
                    high_24h: None,
                    low_24h: None,
                    vol_24h: Some(t.volume),
                    change_24h: Some(t.price_change_percent),
                    exchange: "binance".to_string(),
                    timestamp: TickerInfo::now_timestamp(),
                })
                .collect();
            Ok(result)
        }
        "okx" => {
            let api = okx::okx_api::OkxApi::new();
            let data = api.get_all_tickers().await?;
            let result = data
                .into_iter()
                .map(|t| TickerInfo {
                    symbol: t.instrument_id,
                    last_price: t.last_price,
                    high_24h: Some(t.high_24h),
                    low_24h: Some(t.low_24h),
                    vol_24h: Some(t.volume_24h),
                    change_24h: t.change_24h_pct,
                    exchange: "okx".to_string(),
                    timestamp: TickerInfo::now_timestamp(),
                })
                .collect();
            Ok(result)
        }
        "bitget" => {
            let api = bitget::bitget_api::BitgetApi::new();
            let data = api.get_all_tickers().await?;
            let result = data
                .into_iter()
                .map(|t| TickerInfo {
                    symbol: t.symbol,
                    last_price: t.last_price,
                    high_24h: Some(t.high_24h),
                    low_24h: Some(t.low_24h),
                    vol_24h: Some(t.base_volume),
                    change_24h: Some(t.change_24h_percent),
                    exchange: "bitget".to_string(),
                    timestamp: TickerInfo::now_timestamp(),
                })
                .collect();
            Ok(result)
        }
        "bybit" => {
            let api = bybit::bybit_api::BybitApi::new();
            let data = api.get_all_tickers().await?;
            let result = data
                .into_iter()
                .map(|t| TickerInfo {
                    symbol: t.symbol,
                    last_price: t.last_price,
                    high_24h: Some(t.high_price_24h),
                    low_24h: Some(t.low_price_24h),
                    vol_24h: Some(t.volume_24h),
                    change_24h: Some(t.price_24h_pct),
                    exchange: "bybit".to_string(),
                    timestamp: TickerInfo::now_timestamp(),
                })
                .collect();
            Ok(result)
        }
        _ => Err(anyhow!("unsupported exchange: {}", exchange)),
    }
}
