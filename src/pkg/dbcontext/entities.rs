use sqlx::FromRow;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, FromRow)]
pub struct SymbolKlineData {
    pub symbol: String,
    pub interval: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub open_time: DateTime<Utc>,
    pub volume: f64,
    pub trade_count: i64,
}

pub fn clean_symbol(raw: &str) -> String {
    let symbol = raw.to_uppercase();

    let suffixes = [
        "-USDT-SWAP", "-USDT", "-USD-SWAP", "-USD", "-PERP", "-FUTURE", "-SWAP",
    ];

    for suffix in suffixes.iter() {
        if symbol.ends_with(suffix) {
            return symbol.trim_end_matches(suffix).to_string();
        }
    }

    let quote_assets = ["USDT", "USD", "BUSD", "USDC", "TUSD", "DAI", "USDT_UMCBL"];
    for quote in quote_assets.iter() {
        if symbol.ends_with(quote) {
            return symbol.trim_end_matches(quote).to_string();
        }
    }

    symbol
}