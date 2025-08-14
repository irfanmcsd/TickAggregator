use async_trait::async_trait;
use anyhow::{Result, anyhow};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::pkg::exchanges::bybit::rate_limited_client::RateLimitedClient;
use crate::pkg::exchanges::exchange_entities::{BybitTickerInfo, TickerInfo};
use crate::pkg::exchanges::exchange::ExchangeApi;

pub struct BybitApi {
    client: RateLimitedClient,
}

impl BybitApi {
    pub fn new() -> Self {
        Self {
            client: RateLimitedClient::new(None),
        }
    }
}

#[async_trait]
impl ExchangeApi for BybitApi {
    async fn get_all_tickers(&self) -> Result<Vec<TickerInfo>> {
        let url = "https://api.bybit.com/v5/market/tickers?category=linear";
        let req = reqwest::Client::new()
            .get(url)
            .timeout(std::time::Duration::from_secs(10))
            .build()?;

        let resp = self.client.send_with_retry(req, 1).await?;
        let status = resp.status();
        let bytes = resp.bytes().await?;

        if !status.is_success() {
            let body = String::from_utf8_lossy(&bytes);
            return Err(anyhow!("Non-200 response: {} - {}", status.as_u16(), body));
        }

        // Deserialize into Bybit's specific ticker format
        let bybit_tickers: Vec<BybitTickerInfo> = serde_json::from_slice(&bytes)?;

        // Single timestamp for all tickers
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Convert into standard TickerInfo
        let standard_tickers = bybit_tickers
            .into_iter()
            .map(|b| TickerInfo {
                symbol: b.symbol,
                last_price: b.last_price,
                high_24h: Some(b.high_price_24h),
                low_24h: Some(b.low_price_24h),
                vol_24h: Some(b.volume_24h),
                change_24h: Some(b.price_24h_pct),
                exchange: "Bybit".to_string(),
                timestamp: now,
            })
            .collect();

        Ok(standard_tickers)
    }

    fn name(&self) -> &str {
        "Bybit"
    }

     /*pub async fn get_ticker_info(&self, symbol: &str) -> Result<BybitTickerInfo> {
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
    }*/
}
