
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct TickerInfo {
    pub symbol: String,
    pub last_price: String,
    //pub high_24h: Option<String>,
    //pub low_24h: Option<String>,
    pub vol_24h: Option<String>,
    //pub change_24h: Option<String>,
    //pub exchange: String,
    //pub timestamp: u128, // millis since epoch
}

// BinanceTickerInfo
#[derive(Debug, Deserialize)]
pub struct BinanceTickerInfo {
    pub symbol: String,
    //#[serde(rename = "priceChangePercent")]
    //pub price_change_percent: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    pub volume: String,
}

// BitgetTickerInfo

#[derive(Debug, Deserialize)]
pub struct BitgetTickerInfo {
    pub symbol: String, // e.g., BTCUSDT
    #[serde(rename = "last")]
    pub last_price: String,
    //#[serde(rename = "high24h")]
    //pub high_24h: String,
    //#[serde(rename = "low24h")]
    //pub low_24h: String,
    //#[serde(rename = "changePercent")]
    //pub change_24h_percent: String,
    #[serde(rename = "baseVolume")]
    pub base_volume: String,
}

// BybitTickerInfo

#[derive(Debug, Deserialize)]
pub struct BybitTickerInfo {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    //#[serde(rename = "price24hPcnt")]
    //pub price_24h_pct: String,
    //#[serde(rename = "highPrice24h")]
    //pub high_price_24h: String,
    //#[serde(rename = "lowPrice24h")]
    //pub low_price_24h: String,
    #[serde(rename = "volume24h")]
    pub volume_24h: String,
}

// OkxTickerInfo
#[derive(Debug, Deserialize)]
pub struct OKXTickerInfo {
    #[serde(rename = "instId")]
    pub instrument_id: String,
    #[serde(rename = "last")]
    pub last_price: String,
    //#[serde(rename = "high24h")]
    //pub high_24h: String,
    //#[serde(rename = "low24h")]
    //pub low_24h: String,
    #[serde(rename = "vol24h")]
    pub volume_24h: String,
    //#[serde(rename = "change24h")]
    //pub change_24h_pct: Option<String>, // may not be provided, so optional
}

