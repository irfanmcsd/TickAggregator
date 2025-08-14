use crate::pkg::exchanges::bybit::rate_limited_client::RateLimitedClient;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct BybitTickerInfo {
    pub symbol: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    #[serde(rename = "price24hPcnt")]
    pub price_24h_pct: String,
    #[serde(rename = "highPrice24h")]
    pub high_price_24h: String,
    #[serde(rename = "lowPrice24h")]
    pub low_price_24h: String,
    #[serde(rename = "prevPrice24h")]
    pub prev_price_24h: String,
    #[serde(rename = "turnover24h")]
    pub turnover_24h: String,
    #[serde(rename = "volume24h")]
    pub volume_24h: String,
}

#[derive(Debug, Deserialize)]
struct BybitResponse<T> {
    retCode: i32,
    retMsg: String,
    result: T,
}

#[derive(Debug, Deserialize)]
struct ResultList<T> {
    list: Vec<T>,
}

pub struct BybitApi {
    client: RateLimitedClient,
}

impl BybitApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }

    pub async fn get_all_tickers(&self) -> Result<Vec<BybitTickerInfo>> {
        let url = "https://api.bybit.com/v5/market/tickers?category=linear";
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

        let tickers: Vec<BybitTickerInfo> = serde_json::from_slice(&bytes)?;
        Ok(tickers)
    }

    pub async fn get_ticker_info(&self, symbol: &str) -> Result<BybitTickerInfo> {
        let url = format!(
            "https://api.bybit.com/v5/market/ticker?category=linear&symbol={}",
            symbol
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: BybitTickerInfo = self
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
