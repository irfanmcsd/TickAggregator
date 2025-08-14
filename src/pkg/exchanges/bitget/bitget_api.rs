use crate::pkg::exchanges::bitget::rate_limited_client::RateLimitedClient;
use anyhow::{Context, Result, anyhow};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct BitgetTickerInfo {
    pub symbol: String, // e.g., BTCUSDT
    #[serde(rename = "last")]
    pub last_price: String,
    #[serde(rename = "high24h")]
    pub high_24h: String,
    #[serde(rename = "low24h")]
    pub low_24h: String,
    #[serde(rename = "open24h")]
    pub open_24h: String,
    #[serde(rename = "changePercent")]
    pub change_24h_percent: String,
    #[serde(rename = "baseVolume")]
    pub base_volume: String,
    #[serde(rename = "quoteVolume")]
    pub quote_volume: String,
    #[serde(rename = "ts")]
    pub timestamp: String,
}

#[derive(Debug, Deserialize)]
struct BitgetResponse<T> {
    code: String,
    msg: String,
    data: T,
}

pub struct BitgetApi {
    client: RateLimitedClient,
}

impl BitgetApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }

    pub async fn get_all_tickers(&self) -> Result<Vec<BitgetTickerInfo>> {
        let url = "https://api.bitget.com/api/mix/v1/market/tickers?productType=umcbl";
        let req = reqwest::Client::new()
            .get(url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let resp = self.client.send_with_retry(req, 1).await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            return Err(anyhow!("Non-200 response: {} - {}", status.as_u16(), body));
        }

        let tickers: Vec<BitgetTickerInfo> = serde_json::from_slice(&bytes)?;
        Ok(tickers)
    }

    pub async fn get_ticker_info(&self, symbol: &str) -> Result<BitgetTickerInfo> {
        let url = format!(
            "https://api.bitget.com/api/v2/market/ticker?symbol={}",
            symbol
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: BitgetTickerInfo = self
            .client
            .send_with_retry(req, 1)
            .await
            .context("Send failed")?
            .error_for_status()
            .context("Bad status")?
            .json()
            .await
            .context("JSON parse failed")?;

        Ok(ticker)
    }
}
