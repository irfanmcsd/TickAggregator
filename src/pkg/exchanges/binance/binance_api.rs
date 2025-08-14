use crate::pkg::exchanges::binance::rate_limited_client::RateLimitedClient;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::time::Duration;



#[derive(Debug, Deserialize)]
pub struct TickerInfo {
    pub symbol: String,
    #[serde(rename = "priceChangePercent")]
    pub price_change_percent: String,
    #[serde(rename = "lastPrice")]
    pub last_price: String,
    pub volume: String,
}

pub struct BinanceApi {
    client: RateLimitedClient,
}

impl BinanceApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }

    pub async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>> {
        let url = "https://fapi.binance.com/fapi/v1/ticker/24hr";
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

        let tickers: Vec<TickerInfo> = serde_json::from_slice(&bytes)?;
        Ok(tickers)
    }

     pub async fn get_ticker_info(&self, symbol: &str) -> Result<TickerInfo> {
        let url = format!(
            "https://fapi.binance.com/fapi/v1/ticker/24hr?symbol={}",
            symbol
        );
        let req = reqwest::Client::new()
            .get(&url)
            .timeout(Duration::from_secs(10))
            .build()?;

        let ticker: TickerInfo = self
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
