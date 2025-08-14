use crate::pkg::exchanges::okx::rate_limited_client::RateLimitedClient;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct OKXTickerInfo {
    #[serde(rename = "instId")]
    pub instrument_id: String,
    #[serde(rename = "last")]
    pub last_price: String,
    #[serde(rename = "open24h")]
    pub open_24h: String,
    #[serde(rename = "high24h")]
    pub high_24h: String,
    #[serde(rename = "low24h")]
    pub low_24h: String,
    #[serde(rename = "vol24h")]
    pub volume_24h: String,
    #[serde(rename = "change24h")]
    pub change_24h_pct: Option<String>, // may not be provided, so optional
}

#[derive(Debug, Deserialize)]
struct OKXResponse<T> {
    code: String,
    msg: Option<String>,
    data: T,
}

pub struct OkxApi {
    client: RateLimitedClient,
}

impl OkxApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }

    pub async fn get_all_tickers(&self) -> Result<Vec<OKXTickerInfo>> {
        let url = "https://www.okx.com/api/v5/market/tickers?instType=SWAP";
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

        let tickers: Vec<OKXTickerInfo> = serde_json::from_slice(&bytes)?;
        Ok(tickers)
    }

    pub async fn get_ticker_info(&self, inst_id: &str) -> Result<OKXTickerInfo> {
        let url = format!(
            "https://www.okx.com/api/v5/market/ticker?instId={}",
            inst_id
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: OKXTickerInfo = self
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
